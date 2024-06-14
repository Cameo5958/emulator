use std::sync::mpsc::{self, Reciever, Sender};
use std::f32::consts::PI;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use capl::{SampleFormat, SampleRate, StreamConfig};
use crate::emulator::Emulator;

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
    memory_bus: &MemoryBus,
    frequency: f32,
}

impl SquareWaveChannel {
    pub fn new(has_sweep: bool, offset: u16, memory_bus: &MemoryBus) -> Self {
        SquareWaveChannel { enabled: true, offset: offset, time: 0, has_sweep: has_sweep, 
                            sweep_options: Some(SweepUnit {time: 0, direction: false, shift: 0}),
                            memory_bus: memory_bus, frequency: 0 }
    }

    fn get_sample(&mut self) -> f32 {
        if !self.enabled {
            return 0.0;
        }

        let nr11 = self.memory_bus.read_byte(self.offset + SOUND_LENGTH_OFFSET);
        let nr12 = self.memory_bus.read_byte(self.offset + VOLUME_OFFSET);
        let nr13 = self.memory_bus.read_byte(self.offset + FREQ_LOW_OFFSET);
        let nr14 = self.memory_bus.read_byte(self.offset + FREQ_HIGH_OFFSET);

        let raw_frequency = 2048.0 - ((nr14 as u16 & 0b111) << 8 | nr13 as u16) as f32;
        self.frequency = 131072.0 / raw_frequency;

        if self.has_sweep {
            self.update_sweep();
        }

        let volume = (nr12 >> 4) as f32 / 15.0;

        let duty = match (nr11 >> 6) & 0b11 {
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
        if let Some(ref mut sweep) = self.sweep_unit {
            let nr10 = self.memory_bus.read_byte(self.offset);
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
}

struct WaveformChannel {
    enabled: bool, 
    dac_enabled: bool,
    frequency: f32,
    volume: f32,
    wave_ram: [u8; 16],
    wave_position: usize,
    time: f32,
    memory_bus: &MemoryBus,
}

impl<'a> WaveformChannel<'a> {
    pub fn new(memory_bus: &'a MemoryBus) -> Self {
        WaveformChannel {
            enabled: false,
            dac_enabled: false,
            frequency: 0.0,
            volume: 0.0,
            wave_ram: [0; 16],
            wave_position: 0,
            time: 0.0,
            memory_bus,
        }
    }

    pub fn get_sample(&mut self) -> f32 {
        if !self.enabled || !self.dac_enabled {
            return 0.0;
        }

        // Read the necessary registers
        let nr32 = self.memory_bus.read_byte(NR32);
        self.volume = ((nr32 >> 5) & 0b11) as f32 / 3.0; // Volume is 0, 1/4, 1/2, or 3/4

        // Calculate frequency based on NR33 and NR34
        let nr34 = self.memory_bus.read_byte(NR34);
        let raw_frequency = 2048 - (((nr34 & 0b111) as u16) << 8 | self.memory_bus.read_byte(NR33) as u16);
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
}

pub(crate) struct APU {
    ch1: SquareWaveChannel,
    ch2: SquareWaveChannel,
    ch3: WaveformChannel,
    ch4: NoiseChannel,
}

impl APU {
    pub fn new(em: &Emulator) -> Self {
        APU {
            ch1: SquareWaveChannel::new( true, 0xFF10, &em.mem),
            ch2: SquareWaveChannel::new(false, 0xFF15, &em.mem),
            ch3: WaveformChannel::new(&em.mem),
            ch4: NoiseChannel::new(&em.mem),
        }
    }
    
    pub fn generate_audio(&mut self, sender: Sender<f32>) {
        for _ in 0..(SAMPLE_SIZE as usize) {
            let sample = (elf.CH1.get_sample() + self.ch2.get_sample() + self.ch3.get_sample() + self.ch4.get_sample) / 4;
            if sender.send(sample).is_err() { break; }
        }
    }
}