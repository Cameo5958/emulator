// Represent the CPU using a struct

use crate::{
    registers::{FlagsRegister, Registers},
    memory::MemoryBus,
    instructions::{AllRegisters, AllInstructions, FlagChecks, InterruptIDs},
};

pub(crate) struct CPU { 
    registers:  Registers,      // Registers;  registers.rs
    bus:        MemoryBus,      // Memory Bus; memory.rs

    halted:          bool,
}

// Represents the Core Processing Unit's instructions.
impl CPU { 
    pub fn new() -> Self {
        CPU {
            // Initialize state
            registers   : Registers::new(),
            bus         : MemoryBus::new(), 
            halted      : false,
        }
    }

    fn step(&mut self) {
        // Only execute if not halted
        if self.halted { return }

        // Execute byte located at program counter and shift pc 
        let mut instruction_byte = self.bus.read_increment();
        let prefixed = instruction_byte == 0xCB;

        if prefixed {
            // Prefixed instructions (CB xx) are handled differently
            instruction_byte = self.bus.read_increment();
        }
        
        let Some(instruction) = AllInstructions::decode(instruction_byte, prefixed) else {todo!("do nothing!")};
        self.execute(instruction);
    }

    fn execute(&mut self, instruction: AllInstructions) { 
        use AllInstructions::*;

        match instruction {
            // No target registers
            NOP     => { }
            HALT    => { self.halted = true; }
            STOP    => { }
            DI    => { self.bus.ime = false; }
            EI    => { self.bus.ime = true; }
            RLCA    => {
                let a   = self.registers.a;
                let msb = a & 0x80 >> 7;

                self.registers.a = (a << 1) | msb;
                self.registers.f.carry = msb == 1;
                self.registers.f.zero  = self.registers.a == 0;
            },

            RLA    => {
                let a   = self.registers.a;
                let msb = a & 0x80 >> 7;
                let cf  = if self.registers.f.carry {1} else {0};

                self.registers.a = (a << 1) | cf;
                self.registers.f.carry = msb == 1;
                self.registers.f.zero  = self.registers.a == 0;
            },

            RL(target) => {
                let val = self.get_register_u8(target);
                let msb = val & 0x80 >> 7;
                let cf  = if self.registers.f.carry {1} else {0};

                self.set_register_u8(target, (a << 1) | cf);
                self.registers.f.carry = msb == 1;
                self.registers.f.zero  = self.get_register_u8(target) == 0;
            }

            RRCA    => { 
                let a   = self.registers.a;
                let lsb = a & 0x1;

                self.registers.a = (a >> 1) | lsb;
                self.registers.f.carry = lsb == 1;
                self.registers.f.zero  = self.registers.a == 0;
            },

            RRA    => { 
                let a   = self.registers.a;
                let lsb = a & 0x1;
                let cf  =  if self.registers.f.carry {8} else {0};

                self.registers.a = (a >> 1) | cf;
                self.registers.f.carry = lsb == 1;
                self.registers.f.zero  = self.registers.a == 0;
            },
            
            RR     => {
                let val = self.get_register_u8(target);
                let lsb = val & 0x1;
                let cf = if self.registers.f.carry {8} else {0};

                self.set_register_u8(target, (a >> 1 | cf));
                self.registers.f.carry = lsb == 1;
                self.reigsters.f.zero  = self.get_register_u8(target) == 0;
            }

            DAA    => { 

            },
            CPL    => { self.registers.a = !self.registers.a; }, 
            SCF    => { self.registers.f.carry = true; },
            CCF    => { self.registers.f.carry = !self.registers.f.carry; },
            
            // One target register
            ADD(target) => {
                let value = self.get_register_u8(target);
                self.registers.a = self.add(value, false);
            },
            
            ADD16(target) => {
                let value = self.get_register_u16(target);
                let new_value = self.add_16(value, false);
                self.registers.set_hl(new_value);
            },

            ADC(target) => {
                let value = self.get_register_u8(target);
                self.registers.a = self.add(value, true);
            },

            ADC16(target) => {
                let value = self.get_register_u16(target);
                let new_value = self.add_16(value, true);
                self.registers.set_hl(new_value);
            },

            SUB(target) => {
                let value = self.get_register_u8(target);
                self.registers.a = self.sub(value, false);
            },

            SBC(target) => {
                let value = self.get_register_u8(target);
                self.registers.a = self.sub(value, true);
            },

            AND(target) => {
                let value = self.get_register_u8(target);
                self.registers.a = self._and(value);
            },

            OR(target) => {
                let value = self.get_register_u8(target);
                self.registers.a = self._or(value);
            },

            XOR(target) => {
                let value = self.get_register_u8(target);
                self.registers.a = self._xor(value);
            },

            CP(target) => {
                let value = self.get_register_u8(target);
                self.sub(value, false);
            },

            INC(target) => {
                let value = self.inc(self.get_register_u8(target));
                self.set_register_u8(target, value);
            },

            INC16(target) => {
                let value = self.inc_16(self.get_register_u16(target));
                self.set_register_u16(target, value);
            },

            DEC(target) => {
                let value = self.dec(self.get_register_u8(target));
                self.set_register_u8(target, value);
            },

            DEC16(target) => {
                let value = self.dec_16(self.get_register_u16(target));
                self.set_register_u16(target, value);
            },

            LD(to, from) => {
                self.handle_load(to, from);
            }

            LDI(to, from) => {
                self.handle_load(to, from);
                
                let value = self.inc(self.get_register_u8(from));
                self.set_register_u8(from, value);
            }

            LDD(to, from) => {
                self.handle_load(to, from);

                let value = self.dec(self.get_register_u8(from));
                self.set_register_u8(from, value);
            }

            LDH(to, from) => {
                // Just a different name
                self.handle_load(to, from);
            }

            LDHLSP(from) => {
                // Literally ONE CASE like GO AWAY 
                let value = self.get_register_u16(from);
                self.registers.set_hl(value);
            }

            // Jumps
            JP(cond, to) => {
                self.bus.jump( self.get_register_u16(to), self.get_cond_met(cond) );
            }

            JR(cond, to) => {
                self.bus.jump( self.bus.pc + self.get_register_u8(to) as u16, self.get_cond_met(cond) );
            }

            CALL(cond, to) => {
                // Don't do anything if the condition isn't met
                if !self.get_cond_met(cond) { return; }
                // Push current program counter onto stack
                self.bus.push(self.bus.pc);
                self.bus.jump( self.get_register_u16(to), true);
            },

            RET(cond) => {
                // Don't return unless condition is met
                if self.get_cond_met(cond) { 
                    // Reset program counter
                    self.bus.pc = self.bus.pop();
                }
            }

            RETI(cond) => {
                if self.get_cond_met(cond) {
                    self.bus.pc = self.bus.pop();
                }
            }

            BIT(_pos, target) => {
                let check = self.get_register_u8(target) >> (7 - _pos) & 0xE == 1;
            }

            SET(_pos, target) => {
                
            }
            
            _ => {}
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
            
            U8 => { self.bus.read_increment() },

            // Relative targets
            _ => { self.bus.read_byte( self.get_rel_loc(target) ) },
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
            
            // U8 => { self.bus.set_byte(self.bus.read_increment(), val); };

            // Relative targets
            _ => { self.bus.write_byte(self.get_rel_loc(target) , val) },
        }
    }

    fn get_register_u16(&self, target: AllRegisters) -> u16 {
        use AllRegisters::*;

        match target {
            AF => { self.registers.get_af() }, BC => { self.registers.get_bc() },
            DE => { self.registers.get_de() }, HL => { self.registers.get_hl() },
            SP => { self.bus.sp },

            U16 => {
                let lsb = self.bus.read_increment() as u16;
                let msb = self.bus.read_increment() as u16;
                (msb << 8) | lsb
            }

            _ => { 0x0 }
        }
    }

    fn set_register_u16(&mut self, target: AllRegisters, value: u16) {
        use AllRegisters::*;
        
        match target {
            AF => { self.registers.set_af(value) }, BC => { self.registers.set_bc(value) },
            DE => { self.registers.set_de(value) }, HL => { self.registers.set_hl(value) },

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
            RFFU8   => { 0xFF00      | self.bus.read_increment() as u16 },
            RSPU8   => { self.bus.sp | self.bus.read_increment() as u16 },

            RU16    => {
                let lower_nibble = self.bus.read_increment() as u16;
                let upper_nibble = self.bus.read_increment() as u16;
                (upper_nibble << 8) | lower_nibble
            }

            _ => { 0x0 }
        }
    }

    fn get_cond_met(&self, condition: FlagChecks) -> bool {
        use FlagChecks::*;

        match condition {
            NotZero  => { !self.registers.f.zero  },
            Zero     => {  self.registers.f.zero  },
            NotCarry => { !self.registers.f.carry },
            Carry    => {  self.registers.f.carry },
            Always   => {  true                   },
        }
    }

    fn add(&mut self, value: u8, carry: bool) -> u8 {
        let extra: u8 = if carry {1} else {0};
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value + extra);
        
        self.registers.set_flags(
            new_value == 0, false, did_overflow, 
            (self.registers.a & 0xF) + (value & 0xF) > 0xF,
        );

        return new_value;
    }

    fn add_16(&mut self, value: u16, carry: bool) -> u16 {
        let extra: u16 = if carry {1} else {0};
        let (new_value, did_overflow) = self.registers.get_hl().overflowing_add(value + extra);

        self.registers.set_flags(
            new_value == 0, false, did_overflow, 
            (self.registers.get_hl() & 0xFF) + (value & 0xFF) > 0xFF,
        );

        return new_value;
    }

    fn sub(&mut self, value: u8, carry: bool) -> u8 {
        let extra: u8 = if carry {1} else {0};
        let (new_value, did_overflow) = self.registers.a.overflowing_sub(value + extra);

        self.registers.set_flags(
            new_value == 0, true, did_overflow,
            (self.registers.a & 0xF).overflowing_sub(value & 0xF).1
        );

        return new_value;
    }

    fn _and(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a & value;

        self.registers.set_flags(new_value == 0, false, true, false);
        return new_value;
    }

    fn _or(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a | value;

        self.registers.set_flags(new_value == 0, false, false, false);
        return new_value;
    }

    fn _xor(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a ^ value;

        self.registers.set_flags(new_value == 0, false, false, false);
        return new_value;
    }

    fn inc(&mut self, value: u8) -> u8 {
        let new_value = value + 1;

        self.registers.set_flags(
            new_value == 0, false, self.registers.f.carry, (new_value & 0xF) == 0
        );

        return new_value;
    }

    fn inc_16(&mut self, value: u16) -> u16 {
        let new_value = value + 1;
        self.registers.set_flags(
            new_value == 0, false, self.registers.f.carry, (new_value & 0xFF) == 0
        );

        return new_value;
    }

    fn dec(&mut self, value: u8) -> u8 {
        let new_value = value - 1;

        self.registers.set_flags(
            new_value == 0, false, self.registers.f.carry, (new_value & 0xF) == 0xF
        );

        return new_value;
    }

    fn dec_16(&mut self, value: u16) -> u16 {
        let new_value = value - 1;
        self.registers.set_flags(
            new_value == 0, false, self.registers.f.carry, (new_value & 0xFF) == 0xFF
        );

        return new_value;
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
        let lsb = self.bus.read_increment() as u16;
        let msb = self.bus.read_increment() as u16;

        let loc = (msb << 8) | lsb;
        let val = self.get_register_u8(from);
        
        self.bus.write_byte(loc, val);
    }

    fn handle_interrupts(&mut self, _id: InterruptIDs) {
        
    }
}