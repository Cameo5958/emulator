use std::rc::Rc;
use std::cell::RefCell;

use std::sync::mpsc::{//self, Receiver, 
    Sender};
use cpal::{
    // traits::{DeviceTrait, HostTrait, StreamTrait},
//    {StreamConfig}
};
use crate::{
    memory::MemoryBus,
};

const SAMPLE_SIZE: f32 = 0.0;
const SAMPLE_RATE: f32 = 44100.0;

const SOUND_LENGTH_OFFSET:   u8 = 0x1;
const VOLUME_OFFSET:         u8 = 0x2;
const FREQ_LOW_OFFSET:       u8 = 0x3;
const FREQ_HIGH_OFFSET:      u8 = 0x4;

struct SweepUnit { time: u8, direction: bool, shift: u8, }

struct SquareWaveChannel {
    enabled: bool,
    offset: u16,

    time: f32,
    has_sweep: bool,
    sweep_options: Option<SweepUnit>,
    memory_bus: Rc<RefCell<MemoryBus>>,
    frequency: f32,
}

enum SquareSettingOffsets {
    SweepReg = 0x0, SoundLen = 0x1, Volume   = 0x2,
    FreqLow  = 0x3, FreqHigh = 0x4,
}

impl SquareWaveChannel {
    pub fn new(has_sweep: bool, offset: u16, memory_bus: Rc<RefCell<MemoryBus>>) -> Self {
        SquareWaveChannel { enabled: true, offset: offset, time: 0.0, has_sweep: has_sweep, 
                            sweep_options: Some(SweepUnit {time: 0,   direction: false, shift: 0}),
                            memory_bus: memory_bus, frequency: 0.0 }
    }

    fn get_sample(&mut self) -> f32 {
        use SquareSettingOffsets::*;
        if !self.enabled {
            return 0.0;
        }

        let raw_frequency = 2048.0 - ((self.get(FreqHigh) as u16 & 0b111) << 8 | self.get(FreqLow) as u16) as f32;
        self.frequency = 131072.0 / raw_frequency;

        if self.has_sweep {
            self.update_sweep();
        }

        let volume = (self.get(Volume) >> 4) as f32 / 15.0;

        let duty = match (self.get(SoundLen) >> 6) & 0b11 {
            0 => 0.125, 1 => 0.25, 2 => 0.5,
            3 => 0.75, _ => 0.5,
        };

        let sample_period = SAMPLE_RATE / self.frequency;
        self.time = (self.time + 1.0) % sample_period;
        if self.time < sample_period * duty {
            volume
        } else {
            -volume
        }    
    }

    fn update_sweep(&mut self) {
        if let Some(ref mut sweep) = self.sweep_options {
            let nr10 = self.get(SquareSettingOffsets::SweepReg);
            let sweep_period = (nr10 >> 4) & 0b111;
            sweep.direction = (nr10 & 0b1000) != 0;
            sweep.shift = nr10 & 0b111;

            if sweep_period > 0 {
                if sweep.time == 0 {
                    sweep.time = sweep_period;
                } else {
                    sweep.time -= 1;
                }

                if sweep.time == 0 {
                    let change = self.frequency / (1 << sweep.shift) as f32;
                    if sweep.direction {
                        self.frequency -= change;
                    } else {
                        self.frequency += change;
                    }
                    sweep.time = sweep_period;
                }
            }
        }
    }

    fn get(&mut self, _type: SquareSettingOffsets) -> u8 { self.memory_bus.borrow().read_byte(self.offset + (_type as u16)) }
}

struct WaveformChannel {
    enabled: bool, 
    dac_enabled: bool,
    frequency: f32,
    volume: f32,
    wave_ram: [u8; 16],
    wave_position: usize,
    time: f32,
    memory_bus: Rc<RefCell<MemoryBus>>,
}

impl WaveformChannel {
    pub fn new(mem: Rc<RefCell<MemoryBus>>) -> Self {
        WaveformChannel {
            enabled: false,
            dac_enabled: false,
            frequency: 0.0,
            volume: 0.0,
            wave_ram: [0; 16],
            wave_position: 0,
            time: 0.0,
            memory_bus: mem,
        }
    }

    pub fn read_wvram(&self, addr:u16) -> u8 { self.wave_ram[addr as usize] }

    pub fn write_wvram(&mut self, addr:u16, val:u8) { self.wave_ram[addr as usize] = val; }

    pub fn get_sample(&mut self) -> f32 {
        if !self.enabled || !self.dac_enabled {
            return 0.0;
        }

        // Read the necessary registers
        let nr32 = self.get(0xFF1C);
        self.volume = ((nr32 >> 5) & 0b11) as f32 / 3.0; // Volume is 0, 1/4, 1/2, or 3/4

        // Calculate frequency based on NR33 and NR34
        let nr34 = self.get(0xFF1E);
        let raw_frequency = 2048 - (((nr34 & 0b111) as u16) << 8 | self.get(0xFF1E) as u16);
        self.frequency = 4194304.0 / (32.0 * raw_frequency as f32);

        // Calculate current waveform sample based on time
        let sample_period = SAMPLE_RATE / self.frequency;
        self.time = (self.time + 1.0) % sample_period;
        let wave_length = self.wave_ram.len() as f32;
        let wave_index = (self.time / sample_period * wave_length) as usize;
        let sample_value = self.wave_ram[wave_index];

        // Adjust sample value based on volume
        let normalized_sample = (sample_value as f32 / 15.0) * self.volume;

        if nr32 & 0b1000 == 0 {
            normalized_sample
        } else {
            -normalized_sample
        }
    }

    fn get(&self, loc: u16) -> u8 { self.memory_bus.borrow().read_byte(loc) }
}

struct NoiseChannel<'a> {
    mem: &'a MemoryBus,
}

impl<'a> NoiseChannel<'a> {
    fn new<'a>(mem: &'a MemoryBus) {
        NoiseChannel { mem: mem }
    }
}

pub(crate) struct APU<'a> {
    ch1: SquareWaveChannel<'a>,
    ch2: SquareWaveChannel<'a>,
    ch3: WaveformChannel<'a>,
    ch4: NoiseChannel<'a>,
}

impl<'a> APU<'a> {
    pub fn new(mem: Rc<RefCell<MemoryBus>>) -> Self {
        APU {
            ch1: SquareWaveChannel::new( true, 0xFF10, mem),
            ch2: SquareWaveChannel::new(false, 0xFF15, mem),
            ch3: WaveformChannel::new(mem),
            ch4: NoiseChannel::new(mem),
        }
    }
    
    pub fn generate_audio(&mut self, sender: Sender<f32>) {
        for _ in 0..(SAMPLE_SIZE as usize) {
            let sample = (self.ch1.get_sample() + self.ch2.get_sample() + self.ch3.get_sample() + self.ch4.get_sample()) / 4.0;
            if sender.send(sample).is_err() { break; }
        }
    }

    pub fn write_wvram(&mut self, addr: u16, val: u8) {
        self.ch3.write_wvram(addr, val)
    }

    pub fn read_wvram(&self, addr: u16) -> u8 {
        self.ch3.read_wvram(addr)
    }
}