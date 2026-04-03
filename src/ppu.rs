const VRAM_SIZE: usize = 1024 * 8; //8KiB
const OAM_SIZE: usize = 0xA0; // FE00-FE9F (160 bytes)

const SCREEN_HEIGHT: usize = 144;
const SCREEN_WIDTH: usize = 160;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Transfer = 3,
}

pub struct Ppu {
    screen: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],

    oam: [u8; OAM_SIZE],
    vram: [u8; VRAM_SIZE],

    lcdc: u8, // FF40
    stat: u8, // FF41 (upper bits writable, lower bits mostly status)
    scy: u8,  // FF42
    scx: u8,  // FF43
    ly: u8,   // FF44 (read-only from CPU POV)
    lyc: u8,  // FF45
    bgp: u8,  // FF47
    obp0: u8, // FF48
    obp1: u8, // FF49
    wy: u8,   // FF4A
    wx: u8,   // FF4B

    mode: Mode,
    dot_counter: u32,
    stat_irq_line: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            screen: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
            oam: [0; OAM_SIZE],
            vram: [0; VRAM_SIZE],

            lcdc: 0x91,
            stat: 0x85, // you can start with 0 and evolve; just be consistent
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0xFC,
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,

            mode: Mode::OamScan,
            dot_counter: 0,
            stat_irq_line: false,
        }
    }

    pub fn get_display(&self) -> &[[u8; 160]; 144] {
        &self.screen
    }

    pub fn read_vram(&self, addr: usize) -> u8 {
        if self.mode == Mode::Transfer {
            // During pixel transfer, VRAM is inaccessible to the CPU
            return 0xFF;
        }
        self.vram[addr]
    }

    pub fn write_vram(&mut self, addr: usize, value: u8) {
        if self.mode == Mode::Transfer {
            // During pixel transfer, VRAM is inaccessible to the CPU
            return;
        }
        self.vram[addr] = value;
    }

    pub fn read_oam(&self, addr: usize) -> u8 {
        if self.mode == Mode::OamScan || self.mode == Mode::Transfer {
            // During OAM scan and pixel transfer, OAM is inaccessible to the CPU
            return 0xFF;
        }
        self.oam[addr]
    }
    //TODO: OAM writes should be ignored during certain PPU modes, and OAM should be written to by DMA transfers, not CPU writes. This is a temporary simplification.
    pub fn write_oam(&mut self, addr: usize, value: u8) {
        if self.mode == Mode::OamScan || self.mode == Mode::Transfer {
            // During OAM scan and pixel transfer, OAM is inaccessible to the CPU
            return;
        }
        self.oam[addr] = value;
    }

    pub fn read_reg(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcdc,
            0xFF41 => {
                // Lower 2 bits = mode, bit2 = coincidence. You can compose here:
                let mode_bits = (self.mode as u8) & 0b11;
                let coincidence_bit = if self.ly == self.lyc { 0b100 } else { 0 };
                (self.stat & 0b0111_1000) | coincidence_bit | mode_bits
            }
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF46 => 0xFF, // often reads as last written / undefined; safe: 0xFF
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => 0xFF,
        }
    }

    pub fn dma_write_oam(&mut self, addr: usize, value: u8) {
        if addr < OAM_SIZE {
            self.oam[addr] = value;
        }
    }

    pub fn write_reg(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF40 => self.lcdc = value,
            0xFF41 => {
                // Bits 0-2 are status (mode + coincidence), typically read-only.
                // Bits 3-6 are interrupt enables (writable).
                // Early mask: keep lower 3 bits, allow 3..6 from value.
                self.stat = (self.stat & 0b0000_0111) | (value & 0b0111_1000);
            }
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => {
                //should be read-only
            }
            0xFF45 => self.lyc = value,
            0xFF46 => {
                // Latch request; actual copy should be performed by Bus/Emu after the write finishes
                //TODO;
            }
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            _ => {}
        }
    }

    fn stat_irq_condition(&self) -> bool {
        let mode_irq = match self.mode {
            Mode::HBlank => (self.stat & 0b0000_1000) != 0,
            Mode::VBlank => (self.stat & 0b0001_0000) != 0,
            Mode::OamScan => (self.stat & 0b0010_0000) != 0,
            Mode::Transfer => false,
        };
        let lyc_irq = (self.stat & 0b0100_0000) != 0 && self.ly == self.lyc;
        mode_irq || lyc_irq
    }

    // Returns (entered_vblank, entered_stat_irq).
    pub fn step(&mut self, cycles: u32) -> (bool, bool) {
        let mut entered_vblank = false;

        if (self.lcdc & 0x80) == 0 {
            self.ly = 0;
            self.mode = Mode::HBlank;
            self.dot_counter = 0;
            self.stat_irq_line = false;
            return (false, false);
        }

        self.dot_counter = self.dot_counter.wrapping_add(cycles);

        while self.dot_counter >= 456 {
            self.dot_counter -= 456;

            if self.ly < 144 {
                self.render_scanline();
            }

            self.ly = self.ly.wrapping_add(1);
            if self.ly == 144 {
                entered_vblank = true;
            }
            if self.ly > 153 {
                self.ly = 0;
            }
        }

        self.mode = if self.ly >= 144 {
            Mode::VBlank
        } else if self.dot_counter < 80 {
            Mode::OamScan
        } else if self.dot_counter < 252 {
            Mode::Transfer
        } else {
            Mode::HBlank
        };

        let stat_now = self.stat_irq_condition();
        let entered_stat_irq = !self.stat_irq_line && stat_now;
        self.stat_irq_line = stat_now;

        (entered_vblank, entered_stat_irq)
    }

    fn render_scanline(&mut self) {
        let mut bg_color_ids = [0u8; SCREEN_WIDTH];
        self.render_scanline_bg(&mut bg_color_ids);
        self.render_scanline_window(&mut bg_color_ids);
        self.render_scanline_sprites(&bg_color_ids);
    }


    fn lcdc(&self) -> u8 {
        self.read_reg(0xFF40)
    }

    fn bg_tilemap_base(&self) -> u16 {
        // LCDC bit 3
        if (self.lcdc() & 0x08) != 0 { 0x9C00 } else { 0x9800 }
    }

    fn bg_window_tiledata_base(&self) -> u16 {
        // LCDC bit 4
        if (self.lcdc() & 0x10) != 0 { 0x8000 } else { 0x8800 }
    }

    fn decode_bgp(&self) -> [u8; 4] {
        // BGP: 2 bit pro shade
        let bgp = self.read_reg(0xFF47);
        [
            (bgp >> 0) & 0b11,
            (bgp >> 2) & 0b11,
            (bgp >> 4) & 0b11,
            (bgp >> 6) & 0b11,
        ]
    }

    fn decode_obj_palette(&self, obp: u8) -> [u8; 4] {
        [
            0,
            (obp >> 2) & 0b11,
            (obp >> 4) & 0b11,
            (obp >> 6) & 0b11,
        ]
    }

    fn render_scanline_bg(&mut self, bg_color_ids: &mut [u8; SCREEN_WIDTH]) {
        let ly = self.read_reg(0xFF44) as usize;
        if ly >= SCREEN_HEIGHT { return; }

        // LCDC bit 0: BG enable (DMG)
        if (self.lcdc() & 0x01) == 0 {
            for x in 0..SCREEN_WIDTH {
                self.screen[ly][x] = 0;
                bg_color_ids[x] = 0;
            }
            return;
        }

        let scy = self.read_reg(0xFF42) as usize;
        let scx = self.read_reg(0xFF43) as usize;
        let palette = self.decode_bgp();

        let map_base = self.bg_tilemap_base();
        let data_base = self.bg_window_tiledata_base();

        for x in 0..SCREEN_WIDTH {
            let world_x = (x + scx) & 0xFF;
            let world_y = (ly + scy) & 0xFF;

            let tile_x = world_x / 8;
            let tile_y = world_y / 8;
            let in_tile_x = world_x % 8;
            let in_tile_y = world_y % 8;

            let tilemap_addr = map_base + (tile_y as u16) * 32 + (tile_x as u16);
            let tile_index = self.vram[(tilemap_addr - 0x8000) as usize];

            let tile_addr = if data_base == 0x8000 {
                0x8000 + (tile_index as u16) * 16
            } else {
                // signed index mode
                let signed = tile_index as i8 as i16;
                0x9000u16.wrapping_add((signed * 16) as u16)
            };

            let row_addr = tile_addr + (in_tile_y as u16) * 2;
            let lo = self.vram[(row_addr - 0x8000) as usize];
            let hi = self.vram[(row_addr + 1 - 0x8000) as usize];

            let bit = 7 - in_tile_x;
            let color_id = (((hi >> bit) & 1) << 1) | ((lo >> bit) & 1); // 0..3

            bg_color_ids[x] = color_id;
            self.screen[ly][x] = palette[color_id as usize];
        }
    }

    fn render_scanline_window(&mut self, bg_color_ids: &mut [u8; SCREEN_WIDTH]) {
        // Skeleton: fill/replace this with full window implementation.
        // - Enable: LCDC bit 5
        // - Tile map: LCDC bit 6
        // - Tile data area: LCDC bit 4 (same signed/unsigned logic as BG)
        // - Window position: WX (minus 7), WY

        if (self.lcdc() & 0x20) == 0 {
            return;
        }

        let ly = self.ly as usize;
        if ly >= SCREEN_HEIGHT {
            return;
        }

        let wy = self.wy as usize;
        if ly < wy {
            return;
        }

        let wx_screen = self.wx as i16 - 7;
        if wx_screen >= SCREEN_WIDTH as i16 {
            return;
        }
        let start_x = wx_screen.max(0) as usize;

        let map_base = if (self.lcdc() & 0x40) != 0 { 0x9C00 } else { 0x9800 };
        let data_base = self.bg_window_tiledata_base();
        let palette = self.decode_bgp();

        let window_y = ly - wy;
        let tile_y = window_y / 8;
        let in_tile_y = window_y % 8;

        for x in start_x..SCREEN_WIDTH {
            let window_x = (x as i16 - wx_screen) as usize;
            let tile_x = window_x / 8;
            let in_tile_x = window_x % 8;

            let tilemap_addr = map_base + (tile_y as u16) * 32 + (tile_x as u16);
            let tile_index = self.vram[(tilemap_addr - 0x8000) as usize];

            let tile_addr = if data_base == 0x8000 {
                0x8000 + (tile_index as u16) * 16
            } else {
                let signed = tile_index as i8 as i16;
                0x9000u16.wrapping_add((signed * 16) as u16)
            };

            let row_addr = tile_addr + (in_tile_y as u16) * 2;
            let lo = self.vram[(row_addr - 0x8000) as usize];
            let hi = self.vram[(row_addr + 1 - 0x8000) as usize];

            let bit = 7 - in_tile_x;
            let color_id = (((hi >> bit) & 1) << 1) | ((lo >> bit) & 1);
            bg_color_ids[x] = color_id;
            self.screen[ly][x] = palette[color_id as usize];
        }
    }

    fn render_scanline_sprites(&mut self, bg_color_ids: &[u8; SCREEN_WIDTH]) {
        // Skeleton: fill/replace this with full OBJ priority rules.
        // Implemented here as a useful baseline:
        // - 8x8 / 8x16 size via LCDC bit 2
        // - X/Y flip, palette select, OBJ enable
        // Missing for full accuracy:
        // - precise priority vs BG color 0 + attribute bit
        // - 10 sprites per line limit ordering edge cases

        if (self.lcdc() & 0x02) == 0 {
            return;
        }

        let ly = self.ly as i16;
        if !(0..(SCREEN_HEIGHT as i16)).contains(&ly) {
            return;
        }

        let obj_h = if (self.lcdc() & 0x04) != 0 { 16i16 } else { 8i16 };
        let pal0 = self.decode_obj_palette(self.obp0);
        let pal1 = self.decode_obj_palette(self.obp1);

        let mut sprites_on_line = 0;
        let mut obj_written = [false; SCREEN_WIDTH];

        for i in 0..40 {
            if sprites_on_line >= 10 {
                break;
            }

            let base = i * 4;
            let y = self.oam[base] as i16 - 16;
            let x = self.oam[base + 1] as i16 - 8;
            let mut tile = self.oam[base + 2];
            let attr = self.oam[base + 3];

            if ly < y || ly >= y + obj_h {
                continue;
            }
            sprites_on_line += 1;

            let mut row = (ly - y) as u8;
            if (attr & 0x40) != 0 {
                row = (obj_h as u8 - 1) - row;
            }

            if obj_h == 16 {
                tile &= 0xFE;
            }

            let tile_addr = 0x8000u16 + (tile as u16) * 16 + (row as u16) * 2;
            let lo = self.vram[(tile_addr - 0x8000) as usize];
            let hi = self.vram[(tile_addr + 1 - 0x8000) as usize];

            let palette = if (attr & 0x10) != 0 { pal1 } else { pal0 };

            for px in 0..8i16 {
                let sx = x + px;
                if !(0..(SCREEN_WIDTH as i16)).contains(&sx) {
                    continue;
                }

                let bit_index = if (attr & 0x20) != 0 {
                    px as u8
                } else {
                    7 - px as u8
                };

                let color_id = (((hi >> bit_index) & 1) << 1) | ((lo >> bit_index) & 1);
                if color_id == 0 {
                    continue; // transparent OBJ pixel
                }

                let sx_u = sx as usize;
                if obj_written[sx_u] {
                    continue;
                }

                // OBJ priority bit: 1 = behind BG/Window colors 1..3, in front of color 0.
                if (attr & 0x80) != 0 && bg_color_ids[sx_u] != 0 {
                    continue;
                }

                self.screen[ly as usize][sx_u] = palette[color_id as usize];
                obj_written[sx_u] = true;
            }
        }
    }
}
