// // Store all instructions in an enum
// enum Instruction {
//     ADD(ArithmeticTarget),    ADC(ArithmeticTarget),      SUB(ArithmeticTarget),
//     SBC(ArithmeticTarget),    AND(ArithmeticTarget),      OR(ArithmeticTarget),
//     XOR(ArithmeticTarget),    CP(ArithmeticTarget),       INC(ArithmeticTarget),
//     DEC(ArithmeticTarget),    ADDHL,                      ADCHL,
//     SUBHL,                    SBCHL,                      ANDHL,
//     ORHL,                     XORHL,                      CPHL,
//     INCHL,                    DECHL,                      NOP,
//     HALT,                     STOP,                       DI,
//     EI,                       LD(ArithmeticTarget, ArithmeticTarget),
//     LDHL,                     LDSPHL,                     LDI,
//     LDD,                      PUSH(ArithmeticTarget),     POP(ArithmeticTarget),
//     JP,                       JR,                         CALL,
//     RET,                      RETI,                       RST(u8),
//     DAA,                      CPL,                        CCF,
//     SCF,                      RLCA,                       RLA,
//     RRCA,                     RRA,                        RLC(ArithmeticTarget),
//     RL(ArithmeticTarget),     RRC(ArithmeticTarget),      RR(ArithmeticTarget),
//     SLA(ArithmeticTarget),    SRA(ArithmeticTarget),      SWAP(ArithmeticTarget),
//     SRL(ArithmeticTarget),    BIT(u8, ArithmeticTarget),  RES(u8, ArithmeticTarget),
//     SET(u8, ArithmeticTarget),                           // Add more as needed
// }

// enum Registers {
//     A, B, C, D, E, F, H, L
// }

// // All valid target registers for:
// // ADD, 
// enum ArithmeticTarget {
//     A, B, C, D, E, H, L
// }

// impl Instruction {
//     // Decode opcode into instruction
//     fn decode(opcode: u8) -> Option<Instruction> {
//         use ArithmeticTarget::*;
//         use Instruction::*;
//     }
// }

// Represent the CPU using a struct
struct CPU { 
    registers:  Registers,      // Registers;  registers.rs
    bus:        MemoryBus,      // Memory Bus; memory.rs
}

// Represents the Core Processing Unit's instructions.
impl CPU { 
    pub fn new() -> CPU {
        CPU {
            // Initialize state
            let mut registers   = Registers::new();
            let mut bus         = MemoryBus::new(); 
        }
    }

    fn step(&mut self) {
        // Execute byte located at program counter and shift pc 
        let mut instruction_byte = self.bus.read_increment();
        let prefixed = instruction_byte == 0xCB;

        if prefixed {
            // Prefixed instructions (CB xx) are handled differently
            instruction_byte = self.bus.read_increment();
        }
        
        let Some(instruction) = Instruction::from_byte(instruction_byte, prefixed);
        self.execute(instruction);
    }

    fn execute(&mut self, instruction: Instruction) { 
        use Instruction::*;

        match instruction {
            // No target registers
            NOP     => { }
            HALT    => { }
            STOP    => { }
            DI    => { }
            EI    => { }
            RLCA    => {
                let a   = self.registers.a;
                let msb = a & 0x80 >> 7;

                self.registers.a = (a << 1) | msb;
                self.registers.f = msb << CARRY_FLAG_BYTE_POSITION;
            };

            RLA    => {
                let a   = self.registers.a;
                let msb = a & 0x80 >> 7;
                let cf  = self.registers.f.carry;

                self.registers.a = (a << 1) | cf;
                self.registers.f = msb << CARRY_FLAG_BYTE_POSITION;
            };

            RRCA    => { 
                let a   = self.registers.a;
                let lsb = a & 0x1;

                self.registers.a = (a >> 1) | lsb;
                self.registers.f = lsb << CARRY_FLAG_BYTE_POSITION;
            };

            RRA    => { 
                let a   = self.registers.a;
                let lsb = a & 0x1;
                let cf  = self.registers.f.carry;

                self.registers.a = (a >> 1) | cf;
                self.registers.f = ls << CARRY_FLAG_BYTE_POSITION;
            }
            DAA    => { 

            }
            CPL    => { self.registers.a = !self.registers.a; }; 
            SCF    => { self.registers.f.carry = true; };
            CCF    => { self.registers.f.carry = !self.registers.f.carry; };
            
            // One target register
            ADD(target) => {
                let value = get_register_u8(target);
                self.registers.a = self.add(value, false);
            }

            ADC(target) => {
                let value = get_register_u8(target);
                self.registers.a = self.add(value, true);
            }
        }
    }

    fn get_register_u8(&mut self, target: AllRegisters) -> u8 {
        use AllRegisters::*;

        match target {
            // Absolute targets
            A => { self.registers.a }; B => { self.registers.b };
            C => { self.registers.c }; D => { self.registers.d };
            E => { self.registers.e }; F => { self.registers.f };
            H => { self.registers.h }; L => { self.registers.l };
            
            U8 => { self.bus.read_increment(); };

            // Relative targets
            _ => { self.bus.read_byte( self.get_rel_loc() ); };
        }
    }

    fn get_register_u16(&mut self, target: AllRegisters) -> u16 {
        use AllRegisters::*;

        match target {
            AF => { self.registers.get_af(); }; BC => { self.registers.get_bc(); };
            DE => { self.registers.get_de(); }; HL => { self.registers.get_hl(); };
            SP => { self.bus.sp; }

            U16 => {
                let lsb = self.bus.read_increment() as u16;
                let msb = self.bus.read_increment() as u16;
                (msb << 8) | lsb
            }
        }
    }

    fn get_rel_loc(&mut self, target: AllRegisters) {
        use AllRegisters::*;

        match target {
            rAF     => { self.registers.get_af(); };
            rBC     => { self.registers.get_bc(); };
            rDE     => { self.registers.get_de(); };
            rHL     => { self.registers.get_hl(); };
            
            rFFC    => { 0xFF00      | self.registers.c; };
            rFFU8   => { 0xFF00      | self.bus.read_increment(); };
            rSPU8   => { self.bus.sp | self.bus.read_increment(); };

            rU16    => {
                let lower_nibble = self.bus.read_increment() as u16;
                let upper_nibble = self.bus.read_increment() as u16;
                (upper_nibble << 8) | lower_nibble
            }
        }
    }

    fn add(&mut self, value: u8, carry: bool) -> u8 {
        value += carry;
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value);
        
        self.registers.set_flags(
            new_value == 0, false, did_overflow, 
            (self.registers.a & 0xF) + (value & 0xF) > 0xF;
        )

        new_value;
    }

    fn sub(&mut self, value: u8, carry: bool) -> u8 {
        value += carry;
        let (new_value, did_overflow) = self.registers.a.overflowing_sub(value);

        self.registers.set_flags(
            new_value == 0, true, did_overflow,
            (self.registers.a & 0xF).overflowing_sub(value & 0xF).1
        );

        new_value;
    }
}