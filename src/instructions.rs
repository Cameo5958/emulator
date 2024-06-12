// Implement all instructions

// Store instructions in enum
enum Instruction {
    // No target register
    NOP, HALT, STOP, DI, EI, RLCA, RLA,
    RRCA, RRA, DAA, CPL, SCF, CCF,

    // Arithmetic
    ADD(AllRegisters), ADC(AllRegisters), SUB(AllRegisters), SBC(AllRegisters),
    AND(AllRegisters), OR(AllRegisters), XOR(AllRegisters), CP(AllRegisters),
    INC(AllRegisters), DEC(AllRegisters),

    // Loads
    LD(AllRegisters, AllRegisters), LDI(AllRegisters, AllRegisters), LDD(AllRegisters, AllRegisters),
    LDH(AllRegisters, AllRegisters),

    // 16-bit arithmetic
    ADD16(AllRegisters, AllRegisters), INC16(AllRegisters), DEC16(AllRegisters),

    // Jumps
    JP(FlagChecks, AllRegisters), JR(FlagChecks, AllRegisters),
    CALL(FlagChecks, AllRegisters), RET, RETI, RETC(FlagChecks), RST(u8),

    // Bitwise operations
    BIT(AllRegisters), RES(AllRegisters), SET(AllRegisters),
    RLC(AllRegisters), RL(AllRegisters), RRC(AllRegisters), RR(AllRegisters),
    SLA(AllRegisters), SRA(AllRegisters), SWAP(AllRegisters), SRL(AllRegisters),
}

// Store possible targets for instructions
enum AllRegisters { 
    // Absolute targets
    A,  B,  C,  D, 
    E,  F,  H,  L, 
    AF, BC, DE, HL, 
    SP, U8, U16,
    
    // Relative targets
    rAF,   rBC,   rDE, 
    rHL,   rFFC,  rFFU8, 
    rSPU8, rU16, 
}

// Store possible flag checks
enum FlagChecks { NotZero, Zero, NotCarry, Carry, Always }

// Store possible interrupt IDs
enum InterruptIDs {  }

// Implement conversion from byte to instruction
impl Instruction {
    fn decode(byte: u8, prefixed: bool) -> Option<Instruction> {
        if (prefixed) {
            Instruction::from_byte(byte)
        } else {
            Instruction::from_prefixed_byte(byte)
        }
    }

    fn from_byte(byte: u8) -> Option<Instruction> {
        use Instruction::*;
        use AllRegisters::*;

        match byte {
            0x00 => Some(NOP),
            
            0x03 => Some(INC(BC)),
            0x04 => Some(INC(B)),
            0x05 => Some(DEC(B)),
            _ => println!("Implement: {}", byte)
        }
    }

    fn from_prefixed_byte(byte: u8) -> Option<Instruction> {

    }
}
