use crate::{ cpu::CPU, memory::Memory, ppu::PPU, apu::APU, input::Input, };

pub fn run() {
    let mut cpu     = CPU::new();
    let mut memory  = Memory::new();
    let mut ppu     = PPU::new();
    let mut apu     = APU::new();
    let mut input   = Input::new();

    loop {}
}