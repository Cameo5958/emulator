use crate::emulator::Emulator;

const BOOT_ROM: [u8; 0xFF] = [
    0x31, 0xFE, 0xFF, 0xAF, 0x21, 0x4C, 0x01, 0xCD, 0x87, 0x00, 0x31, 0xFE, 0xFF, 0x3E, 0x20, 0xE0,
    0x50, 0x21, 0x00, 0x99, 0x0E, 0x0C, 0x3E, 0x30, 0xE0, 0x40, 0x11, 0xA8, 0x00, 0x1A, 0xCD, 0x95,
    0x00, 0xCD, 0x96, 0x00, 0x13, 0x7D, 0x21, 0x01, 0x99, 0xCD, 0x86, 0x00, 0xAF, 0xC3, 0x52, 0x01,
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
    0x3E, 0x19, 0xEA, 0x10, 0x00, 0x1A, 0xCD, 0x86, 0x00, 0xC9, 0x3E, 0x01, 0xE0, 0x50, 0xC3, 0x00,
    0x01
    ];

const ROM_START:    u16 = 0x0000; const VROM_START:   u16 = 0x4000; const VRAM_START:   u16 = 0x8000; const CRAM_START:   u16 = 0xA000;
const WRAM_START:   u16 = 0xC000; const ERAM_START:   u16 = 0xE000; const UNUSED:       u16 = 0xFEA0; const OAM_START:    u16 = 0xFE00;
const IORG_START:   u16 = 0xFF00; const HRAM_START:   u16 = 0xFF80;

const ROM_END:  u16 = VROM_START - 1; const VROM_END: u16 = VRAM_START - 1; const VRAM_END: u16 = CRAM_START - 1;
const CRAM_END: u16 = WRAM_START - 1; const WRAM_END: u16 = ERAM_START - 1; const ERAM_END: u16 = UNUSED - 1;
const UNUSED_D: u16 = OAM_START - 1;  const OAM_END:  u16 = IORG_START - 1; const IORG_END: u16 = HRAM_START - 1;
const HRAM_END: u16 = 0xFFFE;

pub(crate) struct MemoryBus {
    pub memory:    [u8; 0xFFFF],
    pub pc:        u16,
    pub sp:        u16,
    pub ime:      bool,
    pub sup: &Emulator,
}

impl MemoryBus {
    pub fn new(mb: &Emulator) -> Self {
        pub fn new(mb: &Motherboard) -> Self {
            let mut memory: [u8; 0xFFFF] = [0; 0xFFFF];
            memory[..0xFF].copy_from_slice(&boot_rom);
    
            MemoryBus { memory: memory, pc: 0x0, sp: 0x0, ime: false, sup: mb };
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            // ROM_START...ROM_END => { self.sup.rom[addr as usize] }
            // VROM_START...VROM_END => { }
            VRAM_START...VRAM_END => { self.sup.ppu.read_vram(addr - VRAM_START) }
            CRAM_START...CRAM_END => { }
            UNUSED...UNUSED_D => { 0x00 }
            OAM_START...OAM_END => {}
            _ => self.memory[addr as usize]
        }
        // return self.memory[addr as usize];
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            ROM_START...VROM_END => { } // Can't write to ROM
            VRAM_START...VRAM_END => { self.sup.ppu.write_vram(addr - VRAM_START, val) }
            CRAM_START...CRAM_END => { }
            WRAM_START...WRAM_END => {
                self.memory[addr as usize] = val;
                // Write to ECHO RAM as well
                let eloc = addr - WRAM_START + ERAM_START;
                if eloc <= WRAM_END {
                    self.write_byte(eloc, val);
                }
            }
            UNUSED...UNUSED_D => { } // Can't write to unmapped location
            OAM_START...OAM_END => {}
            _ => self.memory[addr as usize]
        }        
    }

    pub fn write_rom(&mut self) {}

    pub fn read_increment(&self) -> u8 {
        let data = self.memory[self.pc as usize];
        self.pc.wrapping_add(1);

        return data;
    }

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