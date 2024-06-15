use crate::{ cpu::CPU, memory::MemoryBus, ppu::PPU, apu::APU, input::IPU, };
use winit::window::{Window, WindowBuilder}; 
use pixels::{Pixels, SurfaceTexture};

struct ROM {
    bytes: [u8; 0x7FFFFF],
}

impl ROM {
    pub fn load(path: String) -> Self {
        let mut buffer  = Vec::new();
        let mut file    = File::open(path).expect("Invalid ROM path");
        file.read_to_end(&mut buffer).expect("Unable to read ROM");

        ROM { bytes: buffer }
    }
}

pub(crate) struct Screen {
    pub dsp: Window,
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
            dsp: window , pxl: pixels,
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
    pub fn new(_rom: ROM) -> Self{
        let new_mb = Emulator {
            rom: _rom,

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

        self.ipu.process();

        self.process_events();
    }

    fn process_events(&self) {

    }

    pub fn run(&mut self) {
        loop { self.step() }
    }
}

pub fn run() {
    // let mut cpu     = CPU::new();
    // let mut memory  = MemoryBus::new();
    // let mut ppu     = PPU::new();
    // let mut apu     = APU::new();
    // let mut input   = Input::new();

    loop {}
}