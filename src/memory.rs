struct MemoryBus {
    memory: [u8; 0xFFFF]
    pc:      u8;
    sp:      u8;
}

enum MemoryMap {
    rom0, romN, vram, 
    sram, ram,  wram, 
    io,   hram
}

impl MemoryBus {
    fn read_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize];
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        self.memory[addr as usize] = val;
        
        // Write to Echo ram if writing in ram
    }

    fn read_increment(&self) -> u8 {
        self.memory[self.pc as usize];
        self.pc.wrapping_add(1);
    }

    fn push(&mut self, val: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, ((val & 0xFF00) >> 8) as u8);

        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(self.sp, (val & 0xFF) as u8);
    }

    fn pop(&mut self) -> u16 {
        let lsb = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        let msb = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        (msg << 8) | lsb;
    }
}