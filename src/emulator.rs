use crate::{ cpu::CPU, memory::MemoryBus};//, ppu::PPU, apu::APU, input::Input, };

pub fn run() {
    let mut cpu     = CPU::new();
    let mut memory  = MemoryBus::new();
    // let mut ppu     = PPU::new();
    // let mut apu     = APU::new();
    // let mut input   = Input::new();

    loop {}
}