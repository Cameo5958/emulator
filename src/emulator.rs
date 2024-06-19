use std::rc::Rc;
use std::cell::RefCell;

use std::{fs::File, io::Read};

use crate::{ cpu::CPU, memory::MemoryBus, ppu::PPU, input::IPU, timer::Timer}; //, apu::APU };
use winit::{
    window::{Window, WindowBuilder},
    event_loop::{EventLoop, ControlFlow},
    platform::run_return::EventLoopExtRunReturn,
    dpi::LogicalSize,
}; 
use pixels::{Pixels, SurfaceTexture, wgpu::Backends};

pub(crate) struct ROM {
    bytes: Vec<u8>,
    bank: u8,
}

impl ROM {
    pub fn new(path: &str) -> Self {
        let mut buffer  = Vec::new();
        let mut file    = File::open(path).expect("Invalid ROM path");
        file.read(&mut buffer).expect("Unable to read ROM");

        ROM { bytes: buffer, bank: 1, }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => { self.bytes[addr as usize] },
            0x4000..=0x7FFF => { 
                let addr = self.bank as u32 * 0x4000 + (addr - 0x4000) as u32;
                self.bytes[addr as usize]
            }
            _ => 0xFF,
        }
    }

    pub fn switch_bank(&mut self, bank: u8) {
        self.bank = bank;
    }
}

pub(crate) struct Screen {
    pub dsp: Window,
    // pub evt: EventLoop<()>,
    pub pxl: Rc<RefCell<Pixels>>,
}

impl Screen {
    pub fn new(_loop: &EventLoop<()>) -> Self {
        let window = WindowBuilder::new()
            .with_title("Gameboy Emulator")
            .with_inner_size(LogicalSize::new(160, 144))
            .build(_loop)
            .unwrap();

        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new_with_backends(160, 144, surface_texture, pixels::wgpu::Backends::all());

        Screen {
            dsp: window, pxl: Rc::new(RefCell::new(pixels.unwrap())),
        }
    }
}

pub(crate) struct Emulator {
    pub cpu: CPU,
    // // pub apu: APU, 
    pub ppu: PPU,
    pub ipu: IPU,
    pub tmr: Timer,

    // pub dsp: Screen,
}

impl Emulator {
    pub fn new(_loop: &EventLoop<()>) -> Self {
        println!("In emulator::new()");
        let rom = ROM::new("roms\\game.gb");
        let mem = Rc::new(RefCell::new(MemoryBus::new(rom)));
        
        let dsp = Screen::new(_loop);

        Emulator {
            cpu: CPU::new(Rc::clone(&mem)),
            // // apu: apu,
            ppu: PPU::new(Rc::clone(&mem), Rc::clone(&dsp.pxl)),
            ipu: IPU::new(Rc::clone(&mem)),
            tmr: Timer::new(Rc::clone(&mem)),
            
            // dsp: dsp,
        }
    }

    fn step(&mut self) {
        let cycles = self.cpu.step();

        self.tmr.step(cycles);
        // self.ppu.update(cycles);
        // self.apu.update(cycles);
        self.cpu.check_for_interrupts();
    }

    fn process_events(&self) {

    }

    pub fn run(&mut self, _loop: &mut EventLoop<()>) { //&mut self) {
        _loop.run_return(|events, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            // Take a step
            self.step();
            
            // Get events
            self.ipu.poll(&events);

            // Process events
            self.process_events();
        });
    }
}