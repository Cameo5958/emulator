// Implement all AllInstructionss

// Store AllInstructionss in enum
pub(crate) enum AllInstructions {
    // No target register
    NOP, EMPTY, HALT, STOP, DI, EI, RLCA, RLA,
    RRCA, RRA, DAA, CPL, SCF, CCF,

    // Arithmetic
    ADD(AllRegisters), ADC(AllRegisters), SUB(AllRegisters), SBC(AllRegisters),
    AND(AllRegisters), OR(AllRegisters), XOR(AllRegisters), CP(AllRegisters),
    INC(AllRegisters), DEC(AllRegisters),

    // Loads
    LD(AllRegisters, AllRegisters), LDI(AllRegisters, AllRegisters), LDD(AllRegisters, AllRegisters),
    LD16(AllRegisters, AllRegisters),

    // 16-bit arithmetic
    ADD16(AllRegisters), INC16(AllRegisters), DEC16(AllRegisters),

    // Stack pointer operations
    ADDSP, LDSP(AllRegisters),

    // Jumps + stack 
    JP(FlagChecks, AllRegisters), JR(FlagChecks),
    CALL(FlagChecks, AllRegisters), RET(FlagChecks), RETI(FlagChecks), RST(RstParameters),
    PUSH(AllRegisters), POP(AllRegisters),

    // Bitwise operations
    BIT(u8, AllRegisters), RES(u8, AllRegisters), SET(u8, AllRegisters),
    RLC(AllRegisters), RL(AllRegisters), RRC(AllRegisters), RR(AllRegisters),
    SLA(AllRegisters), SRA(AllRegisters), SWAP(AllRegisters), SRL(AllRegisters),
}

// Store possible targets for AllInstructions
#[derive(PartialEq)]
#[derive(Clone, Copy)]
pub(crate) enum AllRegisters { 
    // Absolute targets
    A,  B,  C,  D, 
    E,  F,  H,  L, 
    AF, BC, DE, HL, 
    SP, U8, U16,
    
    // Relative targets
    RAF,   RBC,   RDE, 
    RHL,   RFFC,  RFFU8, 
    SPI8,  RU16, 
}

// Store possible flag checks
#[derive(PartialEq)]
pub(crate) enum FlagChecks { FNZ, FZ, FNC, FC, FA }
pub(crate) enum RstParameters { R00H, R08H, R10H, R18H, R20H, R28H, R30H, R38H }

// Store possible interrupt IDs
pub(crate) enum InterruptIDs { VBlank = 0x01, LCDStat = 0x02, Timer = 0x04, Serial = 0x08, Joypad = 0x10 }

// Implement conversion from byte to AllInstructions
impl AllInstructions {
    pub fn decode(byte: u8, prefixed: bool) -> Option<AllInstructions> {
        if prefixed {
            AllInstructions::from_byte(byte)
        } else {
            AllInstructions::from_prefixed_byte(byte)
        }
    }

    pub fn from_byte(byte: u8) -> Option<AllInstructions> {
        use AllInstructions::*;
        use AllRegisters::*;
        use FlagChecks::*;
        use RstParameters::*;

        Some(match byte {
            0x00 => NOP,            0x01 => LD16(BC, U16),  0x02 => LD(RBC, A),     0x03 => INC16(BC),
            0x04 => INC(B),         0x05 => DEC(B),         0x06 => LD(B, U8),      0x07 => RLCA,
            0x08 => LD16(RU16, SP), 0x09 => ADD16(BC),      0x0A => LD(A, RBC),     0x0B => DEC16(BC),
            0x0C => INC(C),         0x0D => DEC(C),         0x0E => LD(C, U8),      0x0F => RRCA,
            0x10 => STOP,           0x11 => LD16(DE, U16),  0x12 => LD(RDE, A),     0x13 => INC16(DE),
            0x14 => INC(D),         0x15 => DEC(D),         0x16 => LD(D, U8),      0x17 => RLA,
            0x18 => JR(FA),         0x19 => ADD16(DE),      0x1A => LD(A, RDE),     0x1B => DEC16(DE),
            0x1C => INC(E),         0x1D => DEC(E),         0x1E => LD(E, U8),      0x1F => RRA,
            0x20 => JR(FNZ),        0x21 => LD16(HL, U16),  0x22 => LDI(RHL, A),    0x23 => INC16(HL),
            0x24 => INC(H),         0x25 => DEC(H),         0x26 => LD(H, U8),      0x27 => DAA,
            0x28 => JR(FZ),         0x29 => ADD16(HL),      0x2A => LDI(A, RHL),    0x2B => DEC(HL),
            0x2C => INC(L),         0x2D => DEC(L),         0x2E => LD(L, U8),      0x2F => CPL,
            0x30 => JR(FNC),        0x31 => LD16(SP, U16),  0x32 => LDD(RHL, A),    0x33 => INC16(SP),
            0x34 => INC(RHL),       0x35 => DEC(RHL),       0x36 => LD(RHL, U8),    0x37 => SCF,
            0x38 => JR(FC),         0x39 => ADD16(SP),      0x3A => LDD(A, RHL),    0x3B => DEC(SP),
            0x3C => INC(A),         0x3D => DEC(A),         0x3E => LD(A, U8),      0x3F => CCF,

            0x40..=0xBF => {
                let r = match byte & 0x7 { 
                    0x0 => B, 0x1 => C, 0x2 => D,   0x3 => E, 
                    0x4 => H, 0x5 => L, 0x6 => RHL, 0x7 => A,

                    _   => A, // okay thats it. you think you're being funny, mr.compiler,
                              // making me check the default case for something IMPLICITLY
                              // DECLARED to ONLY BE WITHIN IN THE RANGE 0x0..0x7???
                              //
                              // That's it.
                              //
                              // I have bought 340 megatons of firearms 
                              // 400 thousand trained militia
                              // 6 atomic bombs
                              //
                              // And I will use them all on anyone and everyone ever 
                              // associated with the language of rust.
                };

                let op = match (byte - 0x40) >> 0x3 {
                    0x0 => LD(B, r), 0x1 => LD(C, r), 0x2 => LD(D, r),   0x3 => LD(E, r),
                    0x4 => LD(H, r), 0x5 => LD(L, r), 0x6 => LD(RHL, r), 0x7 => LD(A, r),
                    0x8 => ADD(r),   0x9 => ADC(r),   0xA => SUB(r),     0xB => SBC(r),
                    0xC => AND(r),   0xD => XOR(r),   0xE => OR(r),      0xF => CP(r),  

                    _   => NOP,
                };

                if byte == 0x76 {HALT} else {op}
            }

            0xC0 => RET(FNZ),       0xC1 => POP(BC),        0xC2 => JP(FNZ, U16),   0xC3 => JP(FA, U16),
            0xC4 => CALL(FNZ, U16), 0xC5 => PUSH(BC),       0xC6 => ADD(U8),        0xC7 => RST(R00H),    
            0xC8 => RET(FZ),        0xC9 => RET(FA),        0xCA => JP(FZ, U16),    0xCB => NOP, // Should never be referenced
            0xCC => CALL(FZ, U16),  0xCD => CALL(FA, U16),  0xCE => ADC(U8),        0xCF => RST(R08H),
            0xD0 => RET(FNC),       0xD1 => POP(DE),        0xD2 => JP(FNC, U16),   0xD3 => EMPTY, 
            0xD4 => CALL(FNC, U16), 0xD5 => PUSH(DE),       0xD6 => SUB(U8),        0xD7 => RST(R10H),
            0xD8 => RET(FC),        0xD9 => RETI(FA),       0xDA => JP(FC, U16),    0xDB => EMPTY,
            0xDC => CALL(FC, U16),  0xDD => EMPTY,          0xDE => SBC(U8),        0xDF => RST(R18H),
            0xE0 => LD(RFFU8, A),   0xE1 => POP(HL),        0xE2 => LD(RFFC, A),    0xE3 => EMPTY,
            0xE4 => EMPTY,          0xE5 => PUSH(HL),       0xE6 => AND(U8),        0xE7 => RST(R20H),
            0xE8 => ADDSP,          0xE9 => JP(FA, HL),     0xEA => LD(RU16, A),    0xEB => EMPTY,
            0xEC => EMPTY,          0xED => EMPTY,          0xEE => XOR(U8),        0xEF => RST(R28H),
            0xF0 => LD(A, RFFU8),   0xF1 => POP(AF),        0xF2 => LD(A, RFFC),    0xF3 => DI,
            0xF4 => EMPTY,          0xF5 => PUSH(AF),       0xF6 => OR(U8),         0xF7 => RST(R30H),
            0xF8 => LDSP(HL),       0xF9 => LD(SP, HL),     0xFA => LD(A, RU16),    0xFB => EI,
            0xFC => EMPTY,          0xFD => EMPTY,          0xFE => CP(U8),         0xFF => RST(R38H),
        })
    }

    pub fn from_prefixed_byte(byte: u8) -> Option<AllInstructions> {
        use AllInstructions::*;
        use AllRegisters::*;

        let r = match byte & 0x7 {
            0x0 => B, 0x1 => C, 0x2 => D,   0x3 => E,
            0x4 => H, 0x5 => L, 0x6 => RHL, 0x7 => A,

            _   => A,
        };  

        let op = match byte >> 0x3 {
            0x0 => RLC(r), 0x1 => RRC(r), 0x2 => RL(r),   0x3 => RR(r),
            0x4 => SLA(r), 0x5 => SRA(r), 0x6 => SWAP(r), 0x7 => SRL(r),

            0x8 => BIT(0, r), 0x9 => BIT(1, r), 0xA => BIT(2, r), 0xB => BIT(3, r),
            0xC => BIT(4, r), 0xD => BIT(5, r), 0xE => BIT(6, r), 0xF => BIT(7, r),

            0x10 => RES(0, r), 0x11 => RES(1, r), 0x12 => RES(2, r), 0x13 => RES(3, r),
            0x14 => RES(4, r), 0x15 => RES(5, r), 0x16 => RES(6, r), 0x17 => RES(7, r),

            0x18 => SET(0, r), 0x19 => SET(1, r), 0x1A => SET(2, r), 0x1B => SET(3, r),
            0x1C => SET(4, r), 0x1D => SET(5, r), 0x1E => SET(6, r), 0x1F => SET(7, r),   

            _    => NOP,
        };

        Some(op)
    }
}
