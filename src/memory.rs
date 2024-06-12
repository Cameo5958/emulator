pub(crate) struct MemoryBus {
    pub memory: [u8; 0xFFFF],
    pub pc:     u16,
    pub sp:     u16,
    pub ime:   bool,
}

pub(crate) enum MemoryMap {
    Rom0, RomN, Vram, 
    Sram, Ram,  Wram, 
    Io,   Hram
}

impl MemoryBus {
    pub fn new() -> Self {
        MemoryBus {
            memory: [0x0; 0xFFFF],
            pc: 0x0,
            sp: 0x0,
            ime: true,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        return self.memory[addr as usize];
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        self.memory[addr as usize] = val;
        
        // Write to Echo ram if writing in ram
    }

    pub fn read_increment(&self) -> u8 {
        let data = self.memory[self.pc as usize];
        self.pc.wrapping_add(1);

        return data;
    }

    // pub fn call(&mut self, target: u16, condition: bool) {

    // }

    pub fn jump(&mut self, target: u16, condition: bool) {
        if condition { self.pc = target; }
    }

    pub fn push(&mut self, val: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, ((val & 0xFF00) >> 8) as u8);

        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, (val & 0xFF) as u8);
    }

    pub fn pop(&mut self) -> u16 {
        let lsb = self.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        let msb = self.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        return (msb << 8) | lsb;
    }
}