use crate::emulator::Emulator;
use crate::memory::MemoryBus;
use winit::{
    event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use std::sync::{Arc, Mutex};
use tokio::{sync::mpsc, task};


enum KeyConstants {
    KeyUp     = VirtualKeyCode::Up     as isize,
    KeyDown   = VirtualKeyCode::Down   as isize,
    KeyLeft   = VirtualKeyCode::Left   as isize,
    KeyRight  = VirtualKeyCode::Right  as isize,
    KeyA      = VirtualKeyCode::Z      as isize,
    KeyB      = VirtualKeyCode::X      as isize,
    KeyStart  = VirtualKeyCode::Return as isize,
    KeySelect = VirtualKeyCode::RShift as isize,
}

pub enum Button {
    Up = 0, Down = 1, Left  = 2, Right  = 3,
    A  = 4, B    = 5, Start = 6, Select = 7,
}

pub(crate) struct IPU {
    buttons: [bool; 8],
    mem: &MemoryBus,                              
}

impl IPU {
    pub fn new(em: &Emulator) -> Self{
        IPU {
            buttons: [false; 8],
            mem: &em.mem,
        }        
    }

    fn set(&mut self, key: Button, pressed: bool) {
        self.buttons[key] = pressed;
    }

    pub fn poll(event: &Event) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput { input: keyboard_input, .. } => {
                    if let Some(keycode) = keyboard_input.virtual_keycode {
                        let pressed = keyboard_input.state == ElementState::Pressed;
    
                        if let Some(keycode) = keyboard_input.virtual_keycode {
                            let pressed = keyboard_input.state == ElementState::Pressed;
                
                            match keycode {
                                KeyUp     => self.set(Up, pressed),
                                KeyDown   => self.set(Down, pressed),
                                KeyLeft   => self.set(Left, pressed),
                                KeyRight  => self.set(Right, pressed),
                                KeyA      => self.set(A, pressed), 
                                KeyB      => self.set(B, pressed), 
                                KeyStart  => self.set(Start, pressed),
                                KeySelect => self.set(Select, pressed),
                                _ => (),
                            }
                        }
                    }
                }
                _ => (),
            },
            _ => (),
        } 
    }
}