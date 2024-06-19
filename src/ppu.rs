use std::rc::Rc;
use std::cell::RefCell;

use crate::{emulator::Screen, memory::MemoryBus};
use pixels::Pixels;

const VRAM_BEGIN:   u16 = 0x8000;
const OAM_BEGIN:    u16 = 0xFE00;

// #[derive(Copy,Clone)]
// enum TilePixelValue { Zero, One, Two, Three }

// type Tile = [[TilePixelValue; 8]; 8];
// fn empty_tile() -> Tile {
//     [[TilePixelValue::Zero; 8]; 8]
// }

// enum PPUModes { HBlank, VBlank, OamSearch, PixelTransfer }
pub(crate) enum PPUSettings { 
    LCDC = 0xFF40, STAT = 0xFF41, SCY  = 0xFF42, SCX  = 0xFF43,
    LY   = 0xFF44, LYC  = 0xFF45, DMA  = 0xFF46, BGP  = 0xFF47,
    OGP0 = 0xFF48, OGP1 = 0xFF49, WY   = 0xFF4A, WX   = 0xFF4B,
}

pub(crate) struct PPU {
    cycle_count: u16,
    scanline:    u32,
    buffer:    [[u32; 160]; 144],
    pixels:      Rc<RefCell<Pixels>>,
    bus:         Rc<RefCell<MemoryBus>>,
}

impl PPU {
    pub fn new(mem: Rc<RefCell<MemoryBus>>, pxl: Rc<RefCell<Pixels>>) -> Self {
        PPU {
            cycle_count: 0,
            scanline:    0,
            // mode:        PPUModes::HBlank,
            buffer:    [[0x00; 160]; 144],
            pixels:      pxl,
            bus:         mem,
        }
    }

    pub fn read_vram(&self, addr: u16) -> u8 { self.bus.borrow().read_byte(addr + VRAM_BEGIN) }
    pub fn write_vram(&mut self, addr: u16, value: u8) { self.bus.borrow_mut().write_byte(addr + VRAM_BEGIN, value) }

    pub fn read_oam(&self, addr: u16) -> u8 { self.bus.borrow().read_byte(addr + OAM_BEGIN) }
    pub fn write_oam(&mut self, addr: u16, value: u8) { self.bus.borrow_mut().write_byte(addr + OAM_BEGIN, value) }

    pub fn get(&self, setting: PPUSettings) -> u8 { self.bus.borrow().read_byte(setting as u16) }
    pub fn set(&mut self, setting: PPUSettings, val: u8) { self.bus.borrow_mut().write_byte(setting as u16, val); }

    pub fn update(&mut self, cycles: u16) {
        // use PPUModes::*;
        use PPUSettings::*;

        self.cycle_count += cycles;

        while self.cycle_count >= 456 {
            self.cycle_count -= 456;
            let ly = self.get(LY);
            
                 if (ly + 1) % 154 <  144 { self.render_scanline(); } 
            else if (ly + 1) % 154 == 144 { self.bus.borrow_mut().inf |= 0x01; }

            self.set(LY, ly);
        }
    }

    pub fn render_scanline(&mut self) {
        self.render_background();
        self.render_window();
        self.render_sprites();

        self.draw_buffer();
    }

    fn render_background(&mut self) {
        use PPUSettings::*;

        let lcdc = self.get(LCDC);

        if lcdc & 0x01 == 0 { return; }

        let tile_map_area:u16  = if lcdc & 0x08 == 0 { 0x1800 } else { 0x1C00 };
        let tile_data_area:u16 = if lcdc & 0x10 == 0 { 0x0000 } else { 0x0800 };
        let y = self.get(LY).wrapping_add(self.get(LYC));
        let tile_row = (y / 8) as u16 * 32;

        for x in 0u8..160 {
            let scrolled_x = x.wrapping_add(self.get(SCX));
            let tile_column = (scrolled_x / 8) as u16;
            let tile_index = self.read_vram(tile_map_area + tile_row + tile_column);
            let tile_address = tile_data_area + (tile_index as u16 * 16);
            let line = self.read_vram(tile_address + ((y % 8) * 2) as u16);

            let color_bit = 1 << (7 - (scrolled_x % 8));
            let color_id = if line & color_bit != 0 { 1 } else { 0 };
            let color = self.get_color(self.get(BGP), color_id);
            self.buffer[self.get(LY) as usize][x as usize] = color;
        }
    }
    fn render_window(&mut self) {
        use PPUSettings::*;
        let lcdc = self.get(LCDC);

        if lcdc & 0x20 == 0 { return; }

        let ly = self.get(LY);
        let wy = self.get(WY);
        let window_y = wy.wrapping_add(ly);
        if ly < wy || window_y >= 144 { return; }

        let tile_map_area = if lcdc & 0x40 == 0 { 0x1800 } else { 0x1C00 };
        let tile_data_area = if lcdc & 0x10 == 0 { 0x0000 } else { 0x0800 };
        let y = ly.wrapping_sub(wy);
        let tile_row = (y / 8) as u16 * 32;

        for x in 0u8..160 {
            let window_x = x.wrapping_sub(self.get(WX).wrapping_sub(7));
            if window_x >= 160 { continue; }

            let tile_column = (window_x / 8) as u16;
            let tile_index = self.read_vram(tile_map_area + tile_row + tile_column);
            let tile_address = tile_data_area + (tile_index as u16 * 16);
            let line = self.read_vram(tile_address + ((y % 8) * 2) as u16);

            let color_bit = 1 << (7 - (window_x % 8));
            let color_id = if line & color_bit != 0 { 1 } else { 0 };
            let color = self.get_color(self.get(BGP), color_id);
            self.buffer[ly as usize][x as usize] = color;
        }
    }

    fn render_sprites(&mut self) {
        use PPUSettings::*;
        let lcdc = self.get(LCDC);

        if lcdc & 0x02 == 0 { return; }

        let sprite_height = if lcdc & 0x04 == 0 { 8 } else { 16 };

        for sprite in (0..40).rev() {
            let index = (sprite * 4) as u16;
            let y = self.read_oam(index) - 16;
            let x = self.read_oam(index + 1) - 8;
            let tile_index = self.read_oam(index + 2);
            let attributes = self.read_oam(index + 3);

            if self.get(LY) < y as u8 || self.get(LY) >= (y + sprite_height) as u8 { continue; }

            let line = if attributes & 0x40 == 0 {
                self.get(LY) - y as u8
            } else {
                (sprite_height as u8) - 1 - (self.get(LY) - y as u8)
            };

            let tile_address:u16 = 0x0000 + (tile_index as u16 * 16) + (line as u16 * 2);
            let tile_data = self.read_vram(tile_address);

            for tile_x in 0u8..8 {
                let color_bit = 1 << (7 - tile_x);
                let color_id = if tile_data & color_bit != 0 { 1 } else { 0 };

                if color_id == 0 { continue; }

                let pixel_x = if attributes & 0x20 == 0 {
                    x + tile_x
                } else {
                    x + (7 - tile_x) 
                };

                if pixel_x < 0 || pixel_x >= 160 { continue; }

                let palette = if attributes & 0x10 == 0 {
                    self.get(OGP0)
                } else {
                    self.get(OGP1)
                };
                let color = self.get_color(palette, color_id);
                self.buffer[self.get(LY) as usize][pixel_x as usize] = color;
            }
        }
    }

    fn draw_buffer(&mut self) {
        let mut pxs = self.pixels.borrow_mut();
        let frame   = pxs.get_frame();

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % 160) as usize;
            let y = (i / 160) as usize;
            let color = self.buffer[y][x];
            
            pixel[0] = (color >> 16) as u8;
            pixel[1] = (color >> 8)  as u8;
            pixel[2] = (color)       as u8;
            pixel[3] = 0xFF;
        }

        self.pixels.borrow().render().expect("Failed to render frame");
    }

    fn get_color(&self, palette: u8, color_id: u8) -> u32 {
        match (palette >> (color_id * 2)) & 0x03 {
            0 => 0xFFFFFF, 
            1 => 0xAAAAAA,
            2 => 0x555555,
            3 => 0x000000,
            _ => 0x069420,
        }
    }
}