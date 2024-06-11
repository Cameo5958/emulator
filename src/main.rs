mod apu;
mod cpu;
mod emulator;
mod input;
mod instructions;
mod memory;
mod ppu;
mod registers;

fn main() {
    // Initialize and run emulator
    emulator::run()
}
