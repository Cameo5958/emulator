use std::rc::Rc;
use std::cell::RefCell;

use crate::memory::MemoryBus;
use winit::{
    event::{Event, WindowEvent, ElementState, VirtualKeyCode},
    // event_loop::{ControlFlow, EventLoop},
    // window::WindowBuilder,
};

const KeyUp: VirtualKeyCode     = VirtualKeyCode::Up;   
const KeyDown: VirtualKeyCode   = VirtualKeyCode::Down;
const KeyLeft: VirtualKeyCode   = VirtualKeyCode::Left;
const KeyRight: VirtualKeyCode  = VirtualKeyCode::Right;
const KeyA: VirtualKeyCode      = VirtualKeyCode::Z;
const KeyB: VirtualKeyCode      = VirtualKeyCode::X;   
const KeyStart: VirtualKeyCode  = VirtualKeyCode::Return;
const KeySelect: VirtualKeyCode = VirtualKeyCode::RShift;

pub enum Button {
    Up = 0, Down = 1, Left  = 2, Right  = 3,
    A  = 4, B    = 5, Start = 6, Select = 7,
}

pub(crate) struct IPU {
    buttons: [bool; 8],
    mem: Rc<RefCell<MemoryBus>>,                              
}

impl IPU {
    pub fn new(mem: Rc<RefCell<MemoryBus>>) -> Self{
        IPU {
            buttons: [false; 8],
            mem: mem,
        }        
    }

    fn get(&self, key: Button) -> bool {
        self.buttons[key as usize]
    }

    fn set(&mut self, key: Button, pressed: bool) {
        self.buttons[key as usize] = pressed;
    }

    fn update_byte(&self) {
        use Button::*;

        let mut curr = self.mem.borrow().read_byte(0xFF00);

        if curr & 0x10 != 0 { // Directional Keys
            curr |= if self.get(Down)   { 0x8 } else { 0x0 };
            curr |= if self.get(Up)     { 0x4 } else { 0x0 };
            curr |= if self.get(Left)   { 0x2 } else { 0x0 };
            curr |= if self.get(Right)  { 0x1 } else { 0x0 };
        }
        else if curr & 0x20 != 0 { // Button Keys
            curr |= if self.get(Start)  { 0x8 } else { 0x0 };
            curr |= if self.get(Select) { 0x4 } else { 0x0 };
            curr |= if self.get(B)      { 0x2 } else { 0x0 };
            curr |= if self.get(A)      { 0x1 } else { 0x0 };
        }
        else { curr = 0xC0; }

        self.mem.borrow_mut().write_byte(0xFF00, curr);
    }

    pub fn poll(&mut self, event: &Event<()>) {
        use Button::*;
        
        match event {
            Event::WindowEvent { event, .. } => match event {
                // WindowEvent::CloseRequested => {
                //     *control_flow = ControlFlow::Exit;
                // }
                WindowEvent::KeyboardInput { input: keyboard_input, .. } => {
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
                _ => (),
            },
            _ => (),
        } 

        self.update_byte();
    }
}