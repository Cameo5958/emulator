// Implement all AllInstructionss

// Store AllInstructionss in enum
pub(crate) enum AllInstructions {
    // No target register
    NOP, HALT, STOP, DI, EI, RLCA, RLA,
    RRCA, RRA, DAA, CPL, SCF, CCF,

    // Arithmetic
    ADD(AllRegisters), ADC(AllRegisters), SUB(AllRegisters), SBC(AllRegisters),
    AND(AllRegisters), OR(AllRegisters), XOR(AllRegisters), CP(AllRegisters),
    INC(AllRegisters), DEC(AllRegisters),

    // Loads
    LD(AllRegisters, AllRegisters), LDI(AllRegisters, AllRegisters), LDD(AllRegisters, AllRegisters),
    LDH(AllRegisters, AllRegisters), LDHLSP(AllRegisters),

    // 16-bit arithmetic
    ADD16(AllRegisters), ADC16(AllRegisters), INC16(AllRegisters), DEC16(AllRegisters),

    // Jumps
    JP(FlagChecks, AllRegisters), JR(FlagChecks, AllRegisters),
    CALL(FlagChecks, AllRegisters), RET(FlagChecks), RETI(FlagChecks), RST(u8),

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
    RSPU8, RU16, 
}

// Store possible flag checks
pub(crate) enum FlagChecks { NotZero, Zero, NotCarry, Carry, Always }

// Store possible interrupt IDs
pub(crate) enum InterruptIDs {  }

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

        match byte {
            0x00 => Some(NOP),
            
            0x03 => Some(INC(BC)),
            0x04 => Some(INC(B)),
            0x05 => Some(DEC(B)),
            _ => Some(NOP),
        }
    }

    pub fn from_prefixed_byte(byte: u8) -> Option<AllInstructions> {
        use AllInstructions::*;
        use AllRegisters::*;

        let r = match (byte & 0x7) {
            0x0 => B, 0x1 => C, 0x2 => D,   0x3 => E,
            0x4 => H, 0x5 => L, 0x6 => RHL, 0x7 => A,
        };
        let op = match (byte >> 0x3) {
            0x0 => RLC(r), 0x1 => RRC(r), 0x2 => RL(r),   0x3 => RR(r),
            0x4 => SLA(r), 0x5 => SRA(r), 0x6 => SWAP(r), 0x7 => SRL(r),

            0x8 => BIT(0, r), 0x9 => BIT(1, r), 0xA => BIT(2, r), 0xB => BIT(3, r),
            0xC => BIT(4, r), 0xD => BIT(5, r), 0xE => BIT(6, r), 0xF => BIT(7, r),

            0x10 => RES(0, r), 0x11 => RES(1, r), 0x12 => RES(2, r), 0x13 => RES(3, r),
            0x14 => RES(4, r), 0x15 => RES(5, r), 0x16 => RES(6, r), 0x17 => RES(7, r),

            0x18 => SET(0, r), 0x19 => SET(1, r), 0x1A => SET(2, r), 0x1B => SET(3, r),
            0x1C => SET(4, r), 0x1D => SET(5, r), 0x1E => SET(6, r), 0x1F => SET(7, r),   
        };

        return Some(op);
    }
}
