use crate::{ cpu::CPU, memory::MemoryBus, ppu::PPU, apu::APU, input::IPU, };

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

pub(crate) struct Emulator {
    pub rom: ROM,

    pub cpu: CPU,
    pub apu: APU, 
    pub ppu: PPU,
    pub ipu: IPU,

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

            mem: None,
        };

        new_mb.mem = MemoryBus::new(&new_mb);
        new_mb.cpu = CPU::new(&new_mb);
        new_mb.apu = APU::new(&new_mb);
        new_mb.ppu = PPU::new(&new_mb);
        new_mb.ipu = IPU::new(&new_mb);

        new_mb
    }

    pub fn run(&mut self) {

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