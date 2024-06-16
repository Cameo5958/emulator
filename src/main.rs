mod apu;
mod cpu;
mod emulator;
mod input;
mod instructions;
mod memory;
mod ppu;
mod registers;

use emulator::Emulator;

fn main() {
    // Create an emulator
    Emulator::new("pkmn");
    // Initialize and run emulator
    emulator::run()
}
