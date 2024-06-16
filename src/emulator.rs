use std::ops::ControlFlow;

use crate::{ cpu::CPU, memory::MemoryBus, ppu::PPU, apu::APU, input::IPU, };
use winit::window::{Window, WindowBuilder}; 
use pixels::{Pixels, SurfaceTexture};

struct ROM {
    bytes: [u8; 0x7FFFFF],
}

impl ROM {
    pub fn load(path: &str) -> Self {
        let mut buffer  = Vec::new();
        let mut file    = File::open(path).expect("Invalid ROM path");
        file.read_to_end(&mut buffer).expect("Unable to read ROM");

        ROM { bytes: buffer }
    }
}

pub(crate) struct Screen {
    pub dsp: Window,
    pub evt: EventLoop,
    pub pxl: Pixels,
}

impl Screen {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Gameboy Emulator")
            .build(&event_loop)
            .unwrap();

        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(160, 144, surface_texture).unwrap();

        Screen {
            dsp: window, evt: event_loop, pxl: pixels,
        }
    }
}

pub(crate) struct Emulator {
    pub rom: ROM,

    pub cpu: CPU,
    pub apu: APU, 
    pub ppu: PPU,
    pub ipu: IPU,

    pub dsp: Screen,
    pub mem: MemoryBus,
}

impl Emulator {
    pub fn new(query: &str) -> Self{
        let new_mb = Emulator {
            rom: ROM::load(query),

            cpu: None,
            apu: None,
            ppu: None,
            ipu: None,

            dsp: None,
            mem: None,
        };

        new_mb.mem = MemoryBus::new(&new_mb);
        new_mb.cpu = CPU::new(&new_mb);
        new_mb.apu = APU::new(&new_mb);
        new_mb.ppu = PPU::new(&new_mb);
        new_mb.ipu = IPU::new(&new_mb);

        new_mb.dsp = Screen::new();

        new_mb
    }

    fn step(&mut self) {
        let cycles = self.cpu.step();

        self.timer.update(cycles);
        self.ppu.update(&cycles);
        self.apu.update(cycles);
        self.cpu.check_for_interrupts();
    }

    fn process_events(&self) {

    }

    pub fn run(&mut self) {
        self.dsp.event_loop.run_return(|events, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            // Take a step
            self.step();
            
            // Get events
            ipu.poll(&events);

            // Process events
            self.process_events();
        });
        loop { self.step() }
    }
}