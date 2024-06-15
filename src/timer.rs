use crate::emulator::Emulator;

pub(crate) enum TimerPointers { Div = 0xFF04, Tima = 0xFF05, Tma = 0xFF06, Tac = 0xFF07 }

pub(crate) struct Timer {
    memory: &MemoryBus,
    div_counter: u16, 
}

impl Timer {
    pub fn new(em: &Emulator) { 
        Timer {
            memory: &em.mem,
            div_counter: 0x0,
        }
    }

    pub fn step(&mut self, cycles: u16) {
        use TimerPointers::*;

        self.div_counter += self.div_counter.wrapping_add(cycles);

        if self.tac & 0x04 != 0 { 
            if (self.div_counter as usize) % match (self.tac & 0x3) { 
                0b00 => 0x400, 0b01 => 0x00F, 0b10 => 0x040, 0b11 => 0x100, 
            } == 0 {
                let tima = self.memory.read_byte(Tima);
                if tima == 0xFF {
                    self.memory.write_byte(Tima, self.memory.read_byte(Tma));
                    self.request_interrupt();
                } else {
                    self.memory.write_byte(Tima, tima.wrapping_add(1));
                }
            }
        }
    }

    fn request_interrupt(&self) {
        self.memory.inf |= 0x04;
    }

    fn write_div(&mut self) {
        self.memory.write_byte(Div, (div_counter >> 8) as u8);
    }
}