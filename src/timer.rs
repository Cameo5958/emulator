use std::rc::Rc;
use std::cell::RefCell;

use crate::memory::MemoryBus;

pub(crate) enum TimerPointers { Div = 0xFF04, Tima = 0xFF05, Tma = 0xFF06, Tac = 0xFF07 }

pub(crate) struct Timer {
    memory: Rc<RefCell<MemoryBus>>,
    div_counter: u16, 
}

impl Timer {
    pub fn new(mem: Rc<RefCell<MemoryBus>>) -> Self { 
        Timer {
            memory: mem,
            div_counter: 0x0,
        }
    }

    pub fn step(&mut self, cycles: u16) {
        use TimerPointers::*;

        self.div_counter += self.div_counter.wrapping_add(cycles);

        if self.read_byte(Tac) & 0x04 != 0 { 
            if (self.div_counter as usize) % match self.read_byte(Tac) & 0x3 { 
                0b00 => 0x400, 0b01 => 0x00F, 0b10 => 0x040, 0b11 => 0x100, _ => 0x0,
            } == 0 {
                let tima = self.read_byte(Tima);
                if tima == 0xFF {
                    self.write_byte(Tima, self.read_byte(Tma));
                    self.request_interrupt();
                } else {
                    self.write_byte(Tima, tima.wrapping_add(1));
                }
            }
        }

        self.write_byte(Div, self.div_counter as u8);
    }

    fn request_interrupt(&mut self) {
        self.memory.borrow_mut().inf |= 0x04;
    }

    fn read_byte(&self, field: TimerPointers) -> u8{
        self.memory.borrow().read_byte(field as u16)
    }

    fn write_byte(&mut self, field: TimerPointers, to: u8) {
        self.memory.borrow_mut().write_byte(field as u16, to);
    }
}