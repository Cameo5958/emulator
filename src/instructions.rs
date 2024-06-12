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
    BIT(AllRegisters), RES(AllRegisters), SET(AllRegisters),
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
        byte;
        return Some(AllInstructions::NOP);
    }
}
