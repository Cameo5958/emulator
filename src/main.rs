// mod lib;
#![allow(non_snake_case)]

// pub mod apu;
pub mod cpu;
pub mod emulator;
pub mod input;
pub mod instructions;
pub mod memory;
pub mod ppu;
pub mod registers;
pub mod timer;
pub mod utils;

use emulator::Emulator;
use winit::event_loop::EventLoop;

fn main() {
    let mut event_loop = EventLoop::new();

    let mut emulator   = Emulator::new(&event_loop);
    // println!("rom loaded!");
    
    // emulator.run(&mut event_loop);
}