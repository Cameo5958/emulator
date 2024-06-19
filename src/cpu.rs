// Represent the CPU using a struct

use std::rc::Rc;
use std::cell::RefCell;

use crate::{
    instructions::{AllInstructions, AllRegisters, FlagChecks, InterruptIDs, RstParameters}, 
    memory::MemoryBus, 
    registers::{FlagsRegister, Registers}
};

pub(crate) struct CPU { 
    registers:      Registers,      // Registers;  registers.rs
    bus: Rc<RefCell<MemoryBus>>,      // Memory Bus; memory.rs

    halted:      bool,
}

const IE_REGISTER_BYTE_LOCATION: u16 = 0xFFFF;

// Represents the Core Processing Unit's instructions.
impl CPU { 
    pub fn new(mem: Rc<RefCell<MemoryBus>>) -> Self {
        CPU {
            // Initialize state
            registers   : Registers::new(),
            bus         : mem, 
            halted      : false,
        }
    }

    pub fn step(&mut self) -> u16 {
        // Only execute if not halted
        if self.halted { return 0; }

        // Execute byte located at program counter and shift pc 
        let mut instruction_byte = self.bus.borrow_mut().read_increment();
        let prefixed = instruction_byte == 0xCB;

        if prefixed {
            // Prefixed instructions (CB xx) are handled differently
            instruction_byte = self.bus.borrow_mut().read_increment();
        }
        
        let Some(instruction) = AllInstructions::decode(instruction_byte, prefixed) else { todo!("do nothing!") };
        println!("Executing...");

        (self.execute(instruction) * 4).into()
    }

    pub fn check_for_interrupts(&mut self) {        
        use InterruptIDs::*;

        // Check for interrupts
        if self.bus.borrow().ime && self.bus.borrow().inf != 0 {
            // Disable interrupts to prevent double interrupts
            self.bus.borrow_mut().ime = false;

            // Enable CPU so we can process interrupt 
            self.halted = false;

            // Find which events are currently enabled
            let query = self.bus.borrow().inf & self.bus.borrow().read_byte(IE_REGISTER_BYTE_LOCATION);

            // Call handle_interrupts();
             let target = if query & VBlank as u8  != 0 { VBlank  }
                    else if query & LCDStat as u8 != 0 { LCDStat }
                    else if query & Timer as u8   != 0 { Timer   }
                    else if query & Serial as u8  != 0 { Serial  }
                    else                               { Joypad  };

            self.handle_interrupts(target);
        }
    }

    fn execute(&mut self, instruction: AllInstructions) -> u8 { 
        use AllInstructions::*;
        use AllRegisters::*;

        match instruction {
            // DONE
            NOP     => { 1 }
            EMPTY   => { 0 }
            HALT    => { self.halted = true; 1 }
            STOP    => { 1 }
            DI      => { self.bus.borrow_mut().ime = false; 1 }
            EI      => { self.bus.borrow_mut().ime = true; 1 }


            RLCA    => {
                let a   = self.registers.a;
                let msb = a & 0x80 >> 7;

                self.registers.a = (a << 1) | msb;
                self.registers.set_flags(false, false, false, msb == 1);

                1
            },

            RLA    => {
                let a   = self.registers.a;
                let msb = a & 0x80 >> 7;
                let cf  = if self.registers.f.carry {1} else {0};

                self.registers.a = (a << 1) | cf;
                self.registers.set_flags(false, false, false, msb == 1);

                1
            },

            RLC(target) => {
                let val   = self.get_register_u8(target);
                let msb = val & 0x80 >> 7;

                self.set_register_u8(target, (val << 1) | msb);
                self.registers.set_flags((val << 1) | msb == 0, false, false, msb == 1);

                if target == RHL { 4 } else { 2 }
            }

            RL(target) => {
                let val = self.get_register_u8(target);
                let msb = val & 0x80 >> 7;
                let cf  = if self.registers.f.carry {1} else {0};

                self.set_register_u8(target, (val << 1) | cf);
                self.registers.set_flags((val << 1 | cf) == 0, false, false, msb == 1);

                if target == RHL { 4 } else { 2 }
            }

            RRCA    => { 
                let a   = self.registers.a;
                let lsb = a & 0x1;

                self.registers.a = (a >> 1) | lsb;
                self.registers.set_flags(false, false, false, lsb == 1);

                1
            },

            RRA    => { 
                let a   = self.registers.a;
                let lsb = a & 0x1;
                let cf  =  if self.registers.f.carry {0x80} else {0};

                self.registers.a = (a >> 1) | cf;
                self.registers.set_flags(false, false, false, lsb == 1);

                4
            },

            RRC(target)    => { 
                let val   = self.get_register_u8(target);
                let lsb = val & 0x1;

                self.set_register_u8(target, (val >> 1) | lsb);
                self.registers.set_flags((val >> 1) | lsb == 0, false, false, lsb == 1);

                if target == RHL { 4 } else { 2 }
            },
            
            RR(target)=> {
                let val = self.get_register_u8(target);
                let lsb = val & 0x1;
                let cf = if self.registers.f.carry {0x80} else {0};

                self.set_register_u8(target, val >> 1 | cf);
                self.registers.set_flags((val >> 1 | cf) == 0, false, false, lsb == 1);

                2
            }

            DAA => {
                let mut a = self.registers.a;
                if self.registers.f.subtract {
                    if self.registers.f.half_carry {
                        a = a.wrapping_sub(0x06);
                    }
                    if self.registers.f.carry {
                        a = a.wrapping_sub(0x60);
                    }
                } else {
                    if self.registers.f.half_carry || (a & 0x0F) > 0x09 {
                        a = a.wrapping_add(0x06);
                    }
                    if self.registers.f.carry || a > 0x9F {
                        a = a.wrapping_add(0x60);
                    }
                }
            
                self.registers.f.half_carry = false;
                self.registers.f.zero = a == 0;
                self.registers.f.carry |= a > 0x99;
                self.registers.a = a;

                1
            },
            CPL    => { 
                self.registers.a = !self.registers.a; 
                self.registers.f.subtract   = true;
                self.registers.f.half_carry = true;

                1 
            }, 

            SCF    => { self.registers.set_flags(self.registers.f.zero, false, false, true); 1 },
            CCF    => { self.registers.set_flags(self.registers.f.zero, false, false, !self.registers.f.carry); 1 },
            
            // One target register
            ADD(target) => {
                let value = self.get_register_u8(target);
                self.registers.a = self.add(value, false);

                if target == RHL { 2 } else { 1 }
            },
            
            ADD16(target) => {
                let value = self.get_register_u16(target);
                let new_value = self.add_16(value, false);
                self.registers.set_hl(new_value);

                2
            },

            ADC(target) => {
                let value = self.get_register_u8(target);
                self.registers.a = self.add(value, true);

                if target == RHL { 2 } else { 1 }
            },

            ADDSP => {
                let sp = self.bus.borrow().sp;
                let new_value = self.add_i8_flags(sp);
                self.bus.borrow_mut().sp = new_value;

                4
            }

            SUB(target) => {
                let value = self.get_register_u8(target);
                self.registers.a = self.sub(value, false);

                if target == RHL { 2 } else { 1 }
            },

            SBC(target) => {
                let value = self.get_register_u8(target);
                self.registers.a = self.sub(value, true);

                if target == RHL { 2 } else { 1 }
            },

            AND(target) => {
                let value = self.get_register_u8(target);
                self.registers.a = self._and(value);

                if target == RHL { 2 } else { 1 }
            },

            OR(target) => {
                let value = self.get_register_u8(target);
                self.registers.a = self._or(value);

                if target == RHL { 2 } else { 1 }
            },

            XOR(target) => {
                let value = self.get_register_u8(target);
                self.registers.a = self._xor(value);

                if target == RHL { 2 } else { 1 }
            },

            CP(target) => {
                let value = self.get_register_u8(target);
                self.sub(value, false);

                if target == RHL { 2 } else { 1 }
            },

            INC(target) => {
                let value = self.inc(self.get_register_u8(target));
                self.set_register_u8(target, value);

                if target == RHL { 3 } else { 1 }
            },

            INC16(target) => {
                let ogval = self.get_register_u16(target);
                let value = self.inc_16(ogval);
                self.set_register_u16(target, value);

                2
            },

            DEC(target) => {
                let value = self.dec(self.get_register_u8(target));
                self.set_register_u8(target, value);

                if target == RHL { 3 } else { 1 }
            },

            DEC16(target) => {
                let ogval = self.get_register_u16(target);
                let value = self.dec_16(ogval);
                self.set_register_u16(target, value);

                2
            },

            LD(to, from) => {
                self.handle_load(to, from);

                if to == RHL || from == RHL { 2 } else { 1 }
            }

            LDI(to, from) => {
                self.handle_load(to, from);
                self.inchl();

                2
            }

            LDD(to, from) => {
                self.handle_load(to, from);
                self.dechl();

                2
            }

            LD16(to, from) => {
                let value = self.get_register_u16(from);
                self.set_register_u16(to, value);

                3
            }

            LDSP(to) => {
                // to is hl, from is sp + i8
                let sp  = self.bus.borrow().sp;
                let val = self.add_i8_flags(sp);
                self.set_register_u16(to, val);

                3
            }

            // Jumps
            JP(cond, to) => {
                let isHL = to == HL;
                let go   = self.get_cond_met(cond);
                let addr = self.get_register_u16(to);
                self.bus.borrow_mut().jump( addr, go );

                if isHL { 1 } else if go { 4 } else { 3 } 
            }

            JR(cond) => {
                let go   = self.get_cond_met(cond);
                let addr = self.add_i8(self.bus.borrow().pc);
                self.bus.borrow_mut().jump( addr, go );

                if go { 3 } else { 2 }
            }

            CALL(cond, to) => {
                // Don't do anything if the condition isn't met
                if !self.get_cond_met(cond) { return 3; }
                // Push current program counter onto stack
                let addr = self.get_register_u16(to);

                self.bus.borrow_mut().push(self.bus.borrow().pc);
                self.bus.borrow_mut().jump(addr, true);

                6
            },

            RET(cond) => {
                // Don't return unless condition is met
                let isFA = cond == FlagChecks::FA;
                let go   = self.get_cond_met(cond);
                if go { let loc = self.bus.borrow_mut().pop(); self.bus.borrow_mut().jump(loc, true);}

                if isFA { 4 } else if go { 5 } else { 2 }
            }

            RETI(cond) => {
                if self.get_cond_met(cond) {
                    self.bus.borrow_mut().pc = self.bus.borrow_mut().pop();
                    self.bus.borrow_mut().ime = true;
                }

                4
            }

            BIT(_pos, target) => {
                let val = self.get_register_u8(target);
                self.registers.f.zero = (val & (1 << _pos)) == 0;
                self.registers.f.subtract = false;
                self.registers.f.carry = true;
                self.registers.set_flags(val & (1 << _pos) == 0, false, true, self.registers.f.carry);

                if target == RHL { 3 } else { 2 }
            }

            SET(_pos, target) => {
                let bitset:u8 = 0x1 << (7 - _pos);
                let val = self.get_register_u8(target);

                self.set_register_u8(target, val | bitset);

                if target == RHL { 4 } else { 2 }
            }

            RES(_pos, target) => {
                let mask:u8 = !(0x1 << (7 - _pos));
                let val = self.get_register_u8(target);

                self.set_register_u8(target, val & mask);

                if target == RHL { 4 } else { 2 }
            }

            SWAP(target) => {
                let val = self.get_register_u8(target);
                let msb = val >> 4;
                self.set_register_u8(target, msb | val << 4);
                self.registers.set_flags((msb | val << 4) == 0, false, false, false);

                if target == RHL { 4 } else { 2 }
            }

            SLA(target) => {
                let val = self.get_register_u8(target);
                self.registers.set_flags(val << 1 == 0, false, false, val & 0x80 == 0x80);
                
                self.set_register_u8(target, val << 1);

                if target == RHL { 4 } else { 2 }
            }

            SRA(target) => {
                let val = self.get_register_u8(target);
                let sign = val & 0x80;
                self.registers.set_flags(val >> 1 | sign == 0, false, false, val & 0x1 == 1);

                self.set_register_u8(target, val >> 1 | sign);
                
                if target == RHL { 4 } else { 2 }
            }

            SRL(target) => {
                let val = self.get_register_u8(target);
                self.registers.set_flags(val >> 1 == 0, false, false, val & 0x1 == 1);

                self.set_register_u8(target, val >> 1);

                if target == RHL { 4 } else { 2 }
            }

            PUSH(target) => { let reg = self.get_register_u16(target); self.bus.borrow_mut().push(reg); 4 }
            POP(target)  => { let val = self.bus.borrow_mut().pop(); self.set_register_u16(target, val); 3 }

            RST(param) => {
                use RstParameters::*;
                let target: u16 = match param {
                    R00H => 0x00, R08H => 0x08, R10H => 0x10, R18H => 0x18,
                    R20H => 0x20, R28H => 0x28, R30H => 0x30, R38H => 0x38,
                };

                self.bus.borrow_mut().push( self.bus.borrow().pc );
                self.bus.borrow_mut().jump( target, true);

                4
            }
        }
    }

    fn get_register_u8(&self, target: AllRegisters) -> u8 {
        use AllRegisters::*;

        match target {
            // Absolute targets
            A => { self.registers.a }, B => {          self.registers.b },
            C => { self.registers.c }, D => {          self.registers.d },
            E => { self.registers.e }, F => { u8::from(self.registers.f) },
            H => { self.registers.h }, L => {          self.registers.l },
            
            U8      => { self.bus.borrow_mut().read_increment()        },

            // Relative targets
            _ => { self.bus.borrow().read_byte( self.get_rel_loc(target) ) },
        }
    }

    fn set_register_u8(&mut self, target: AllRegisters, val: u8) {
        use AllRegisters::*;

        match target {
            // Absolute targets
            A => { self.registers.a = val }, B => { self.registers.b =                     val  },
            C => { self.registers.c = val }, D => { self.registers.d =                     val  },
            E => { self.registers.e = val }, F => { self.registers.f = FlagsRegister::from(val) },
            H => { self.registers.h = val }, L => { self.registers.l =                     val  },
            
            // U8 => { self.bus.borrow().set_byte(self.bus.borrow_mut().read_increment(), val); };

            // Relative targets
            _ => { self.bus.borrow_mut().write_byte(self.get_rel_loc(target) , val) },
        }
    }

    fn get_register_u16(&mut self, target: AllRegisters) -> u16 {
        use AllRegisters::*;

        match target {
            AF => { self.registers.get_af() }, BC => { self.registers.get_bc() },
            DE => { self.registers.get_de() }, HL => { self.registers.get_hl() },
            SP => { self.bus.borrow().sp },

            U16 => {
                let lsb = self.bus.borrow_mut().read_increment() as u16;
                let msb = self.bus.borrow_mut().read_increment() as u16;
                (msb << 8) | lsb
            }
            SPI8    => { let sp = self.bus.borrow().sp; self.add_i8_flags(sp) },

            _ => { 0x0 }
        }
    }

    fn set_register_u16(&mut self, target: AllRegisters, value: u16) {
        use AllRegisters::*;
        
        match target {
            AF => { self.registers.set_af(value) }, BC   => { self.registers.set_bc(value) },
            DE => { self.registers.set_de(value) }, HL   => { self.registers.set_hl(value) },
            SP => { self.bus.borrow_mut().sp = value; },         
            RU16 => { 
                // Little endian?
                let msb = (value & 0xF0) as u8;
                let lsb = (value & 0xF ) as u8;
                    
                let loc = (self.bus.borrow_mut().read_increment() as u16) | ((self.bus.borrow_mut().read_increment() as u16) << 8);

                self.bus.borrow_mut().write_byte(loc, lsb);
                self.bus.borrow_mut().write_byte(loc + 1, msb);
            },

            _ => {}
        }
    }

    fn get_rel_loc(&self, target: AllRegisters) -> u16 {
        use AllRegisters::*;

        match target {
            RAF     => { self.registers.get_af() },
            RBC     => { self.registers.get_bc() },
            RDE     => { self.registers.get_de() },
            RHL     => { self.registers.get_hl() },
            
            RFFC    => { 0xFF00      | self.registers.c as u16 },
            RFFU8   => { 0xFF00      | self.bus.borrow_mut().read_increment() as u16 },

            RU16    => {
                let lower_nibble = self.bus.borrow_mut().read_increment() as u16;
                let upper_nibble = self.bus.borrow_mut().read_increment() as u16;
                (upper_nibble << 8) | lower_nibble
            }

            _ => { 0x0 }
        }
    }

    fn get_cond_met(&self, condition: FlagChecks) -> bool {
        use FlagChecks::*;

        match condition {
            FNZ => { !self.registers.f.zero  },
            FZ  => {  self.registers.f.zero  },
            FNC => { !self.registers.f.carry },
            FC  => {  self.registers.f.carry },
            FA  => {  true                   },
        }
    }

    fn add_i8_flags(&mut self, base: u16) -> u16 {
        let val  = self.get_register_u8(AllRegisters::U8) as i8;
        let (new_value, did_overflow) = base.overflowing_add(val as u16);

        self.registers.set_flags(false, false, base as u8 + val as u8 > 0xF, did_overflow); 

        new_value
    }

    fn add_i8(&self, base: u16) -> u16 {
        let val  = self.get_register_u8(AllRegisters::U8) as i8;
        let new_value = base.wrapping_add(val as u16);

        new_value
    }

    fn add(&mut self, value: u8, carry: bool) -> u8 {
        let extra: u8 = if carry {1} else {0};
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value + extra);
        
        self.registers.set_flags(
            new_value == 0, false, (self.registers.a & 0xF) + (value & 0xF) > 0xF, did_overflow, 
        );

        new_value
    }

    fn add_16(&mut self, value: u16, carry: bool) -> u16 {
        let extra: u16 = if carry {1} else {0};
        let (new_value, did_overflow) = self.registers.get_hl().overflowing_add(value + extra);

        self.registers.set_flags(
            new_value == 0, false, (self.registers.get_hl() & 0xFF) + (value & 0xFF) > 0xFF, did_overflow, 

        );

        new_value
    }

    fn sub(&mut self, value: u8, carry: bool) -> u8 {
        let extra: u8 = if carry {1} else {0};
        let (new_value, did_overflow) = self.registers.a.overflowing_sub(value + extra);

        self.registers.set_flags(
            new_value == 0, true, (self.registers.a & 0xF).overflowing_sub(value & 0xF).1, did_overflow,

        );

        new_value
    }

    fn _and(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a & value;

        self.registers.set_flags(new_value == 0, false, true, false);
        new_value
    }

    fn _or(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a | value;

        self.registers.set_flags(new_value == 0, false, false, false);
        new_value
    }

    fn _xor(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a ^ value;

        self.registers.set_flags(new_value == 0, false, false, false);
        new_value
    }

    fn inc(&mut self, value: u8) -> u8 {
        let new_value = value + 1;

        self.registers.set_flags(
            new_value == 0, false, (new_value & 0xF) == 0, self.registers.f.carry, 
        );

        new_value
    }

    fn inc_16(&mut self, value: u16) -> u16 {
        let new_value = value + 1;

        new_value
    }

    fn inchl(&mut self) {
        let hlval = self.get_register_u16(AllRegisters::HL);
        let value = self.inc_16(hlval);
        self.set_register_u16(AllRegisters::HL, value);
    }

    fn dec(&mut self, value: u8) -> u8 {
        let new_value = value - 1;

        self.registers.set_flags(
            new_value == 0, false, (new_value & 0xF) == 0xF, self.registers.f.carry, 
        );

        new_value
    }

    fn dec_16(&mut self, value: u16) -> u16 {
        let new_value = value - 1;

        new_value
    }

    fn dechl(&mut self) {
        let hlval = self.get_register_u16(AllRegisters::HL);
        let value = self.dec_16(hlval);
        self.set_register_u16(AllRegisters::HL, value);
    }

    fn handle_load(&mut self, to: AllRegisters, from: AllRegisters) {
        if to == AllRegisters::RU16 { self.handle_relative_load(from); }
        else {
            let val = self.get_register_u8(from);
            self.set_register_u8(to, val);
        }
    }

    fn handle_relative_load(&mut self, from: AllRegisters) {
        // The next two bytes represent the absolute next value
        let lsb = self.bus.borrow_mut().read_increment() as u16;
        let msb = self.bus.borrow_mut().read_increment() as u16;

        let loc = (msb << 8) | lsb;
        let val = self.get_register_u8(from);
        
        self.bus.borrow_mut().write_byte(loc, val);
    }

    fn handle_interrupts(&mut self, inter_type: InterruptIDs) {
        use InterruptIDs::*;

        let loc = match inter_type {
            VBlank  => { 0x40 }
            LCDStat => { 0x48 }
            Timer   => { 0x50 }
            Serial  => { 0x58 }
            Joypad  => { 0x60 }
        } as u16;

        self.bus.borrow_mut().push(self.bus.borrow().pc);
        self.bus.borrow_mut().pc = loc;
        self.bus.borrow_mut().inf &= !(inter_type as u8);
    }
}