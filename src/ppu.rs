const VRAM_SIZE: usize = 1024 * 8; //8KiB
const OAM_SIZE: usize = 0xA0; // FE00-FE9F (160 bytes)

const SCREEN_HEIGHT: usize = 144;
const SCREEN_WIDTH: usize = 160;

const TILE_SIZE: usize = 8 * 8;
const BG_MAP0_OFFSET: usize = 0x1800; // 0x9800 - 0x8000
const BG_MAP1_OFFSET: usize = 0x1C00; // 0x9C00 - 0x8000

const CYCLES_OAM_SCAN: u32 = 80;
const CYCLES_TRANSFER: u32 = 172;
const CYCLES_HBLANK: u32 = 204;
const CYCLES_PER_SCANLINE: u32 = 456;
const VBLANK_START_LINE: u8 = 144;
const VBLANK_END_LINE: u8 = 153;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Transfer = 3,
}

type Tile = [u8; TILE_SIZE];

pub struct Ppu {
    screen: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],

    oam: [u8; OAM_SIZE],
    vram: [u8; VRAM_SIZE],
    tile_data: [Tile; 384],         //0x8000-0x97FF

    lcdc: u8, // FF40
    stat: u8, // FF41 (upper bits writable, lower bits mostly status)
    scy: u8,  // FF42
    scx: u8,  // FF43
    ly: u8,   // FF44 (read-only from CPU POV)
    lyc: u8,  // FF45
    bgp: u8,  // FF47 Background Palette
    obp0: u8, // FF48 OBJ0 Palette
    obp1: u8, // FF49 OBJ1 Palette
    wy: u8,   // FF4A
    wx: u8,   // FF4B
    dma: u8,  // FF46 (DMA source high byte)

    mode: Mode,
    mode_cycles: u32,
}

impl Ppu {
    pub fn new() -> Self {
        let mut this = Self {
            screen: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
            oam: [0; OAM_SIZE],
            vram: [0; VRAM_SIZE],
            tile_data: [[0; TILE_SIZE]; 384],

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
            dma: 0xFF,

            mode: Mode::OamScan,
            mode_cycles: 0,
        };

        // Ensure decoded tile cache matches initial VRAM contents.
        this.decode_all_tiles();

        this
    }

    pub fn get_display(&self) -> &[[u8; 160]; 144] {
        &self.screen
    }

    pub fn read_vram(&self, addr: usize) -> u8 {
        if self.mode == Mode::Transfer {
            // During pixel transfer, VRAM is inaccessible to the CPU
            return 0xFF;
        }
        //normalization of Address, becuz none in membus
        //maybe improve
        match addr {
            0x8000..=0x9FFF => self.vram[addr - 0x8000],
            _ => 0xFF,
        }
    }

    pub fn write_vram(&mut self, addr: usize, value: u8) {
        if self.mode == Mode::Transfer {
            // During pixel transfer, VRAM is inaccessible to the CPU
            return;
        }
        //normalization of Address, becuz none in membus
        //maybe improve
        match addr {
            0x8000..=0x97FF => {
                self.vram[addr - 0x8000] = value;
                self.update_tile_from_vram(addr);
            }
            0x9800..=0x9FFF => self.vram[addr - 0x8000] = value,
            _ => unreachable!(),
        }
    }

    // Decode all tiles from VRAM into the `tile_data` cache.
    fn decode_all_tiles(&mut self) {
        // Each tile occupies 16 bytes in VRAM (2 bytes per row * 8 rows)
        let tiles = self.tile_data.len();
        for tile_index in 0..tiles {
            let base = tile_index * 16; // offset into vram (0..=0x17FF)
            for row in 0..8 {
                let b0 = self.vram[base + row * 2];
                let b1 = self.vram[base + row * 2 + 1];
                for x in 0..8 {
                    let lo = (b0 >> (7 - x)) & 1;
                    let hi = (b1 >> (7 - x)) & 1;
                    self.tile_data[tile_index][row * 8 + x] = lo | (hi << 1);
                }
            }
        }
    }

    // Update the decoded pixels for the tile row affected by a VRAM write at `addr`.
    // `addr` is the full GameBoy address in range 0x8000..=0x97FF.
    fn update_tile_from_vram(&mut self, addr: usize) {
        if !(0x8000..=0x97FF).contains(&addr) {
            return;
        }
        let offset = addr - 0x8000; // 0..=0x17FF
        let tile_index = offset / 16; // which tile (0..383)
        let byte_in_tile = offset % 16; // 0..15
        let row = byte_in_tile / 2; // 0..7

        let base = tile_index * 16;
        let b0 = self.vram[base + row * 2];
        let b1 = self.vram[base + row * 2 + 1];
        for x in 0..8 {
            let lo = (b0 >> (7 - x)) & 1;
            let hi = (b1 >> (7 - x)) & 1;
            self.tile_data[tile_index][row * 8 + x] = lo | (hi << 1);
        }
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

    pub fn dma_write_oam(&mut self, addr: usize, value: u8) {
        if addr < OAM_SIZE {
            self.oam[addr] = value;
        }
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
            0xFF46 => self.dma,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => 0xFF,
        }
    }

    pub fn write_reg(&mut self, addr: u16, value: u8) {
        match addr {
            //TODO: LCD enable Toggle only allowed during VBlank
            0xFF40 => {
                let old_enabled = self.lcd_enabled();
                self.lcdc = value;
                let new_enabled = self.lcd_enabled();

                // Simple DMG-compatible handling:
                // LCD OFF => reset LY/mode timing.
                // LCD ON  => start at line 0 in OAM scan.
                if old_enabled && !new_enabled {
                    self.ly = 0;
                    self.mode = Mode::HBlank;
                    self.mode_cycles = 0;
                } else if !old_enabled && new_enabled {
                    self.ly = 0;
                    self.mode = Mode::OamScan;
                    self.mode_cycles = 0;
                }
            }
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
                // Latch request; actual copy is performed by MemoryBus
                self.dma = value;
            }
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            _ => {}
        }
    }

    pub fn step(&mut self, cycles: u32) -> (bool, bool) {
        if !self.lcd_enabled() {
            return (false, false);
        }

        let mut entered_vblank = false;
        let mut entered_stat_irq = false;

        self.mode_cycles = self.mode_cycles.saturating_add(cycles);

        loop {
            let threshold = match self.mode {
                Mode::OamScan => CYCLES_OAM_SCAN,
                Mode::Transfer => CYCLES_TRANSFER,
                Mode::HBlank => CYCLES_HBLANK,
                Mode::VBlank => CYCLES_PER_SCANLINE,
            };

            if self.mode_cycles < threshold {
                break;
            }

            self.mode_cycles -= threshold;
            let (vblank_now, stat_now) = self.advance_mode_once();
            entered_vblank |= vblank_now;
            entered_stat_irq |= stat_now;
        }

        // LYC coincidence can change as LY advances. Raise STAT if enabled and coincidence is true.
        if self.is_lyc_stat_enabled() && self.ly == self.lyc {
            entered_stat_irq = true;
        }

        (entered_vblank, entered_stat_irq)
    }

    fn advance_mode_once(&mut self) -> (bool, bool) {
        match self.mode {
            Mode::OamScan => {
                self.set_mode(Mode::Transfer);
                (false, false)
            }
            Mode::Transfer => {
                if self.ly < VBLANK_START_LINE {
                    let y = self.ly as usize;
                    let mut bg_color_ids = self.bg_scanline_color_ids(y);
                    self.render_bg_scanline(y, &bg_color_ids);
                    self.render_window_scanline(y, &mut bg_color_ids);
                    self.render_obj_scanline(y, &bg_color_ids);
                }
                let stat_irq = self.set_mode(Mode::HBlank);
                (false, stat_irq)
            }
            Mode::HBlank => {
                self.ly = self.ly.wrapping_add(1);
                if self.ly >= VBLANK_START_LINE {
                    let stat_irq = self.set_mode(Mode::VBlank);
                    (true, stat_irq)
                } else {
                    let stat_irq = self.set_mode(Mode::OamScan);
                    (false, stat_irq)
                }
            }
            Mode::VBlank => {
                self.ly = self.ly.wrapping_add(1);
                if self.ly > VBLANK_END_LINE {
                    self.ly = 0;
                    let stat_irq = self.set_mode(Mode::OamScan);
                    (false, stat_irq)
                } else {
                    (false, false)
                }
            }
        }
    }

    fn set_mode(&mut self, new_mode: Mode) -> bool {
        self.mode = new_mode;
        match new_mode {
            Mode::HBlank => self.is_hblank_stat_enabled(),
            Mode::VBlank => self.is_vblank_stat_enabled(),
            Mode::OamScan => self.is_oam_stat_enabled(),
            Mode::Transfer => false,
        }
    }

    fn lcd_enabled(&self) -> bool {
        (self.lcdc & 0x80) != 0
    }

    fn is_hblank_stat_enabled(&self) -> bool {
        (self.stat & 0x08) != 0
    }

    fn is_vblank_stat_enabled(&self) -> bool {
        (self.stat & 0x10) != 0
    }

    fn is_oam_stat_enabled(&self) -> bool {
        (self.stat & 0x20) != 0
    }

    fn is_lyc_stat_enabled(&self) -> bool {
        (self.stat & 0x40) != 0
    }

    fn bg_scanline_color_ids(&self, y: usize) -> [u8; SCREEN_WIDTH] {
        let mut line = [0u8; SCREEN_WIDTH];
        if y >= SCREEN_HEIGHT || !self.bg_enabled() {
            return line;
        }

        let map_base = self.bg_map_base_offset();
        let scrolled_y = self.scy.wrapping_add(y as u8);
        let tile_row = (scrolled_y as usize) / 8;
        let pixel_row = (scrolled_y as usize) % 8;

        for (x, out) in line.iter_mut().enumerate() {
            let scrolled_x = self.scx.wrapping_add(x as u8);
            let tile_col = (scrolled_x as usize) / 8;
            let pixel_col = (scrolled_x as usize) % 8;

            let map_index = tile_row * 32 + tile_col;
            let tile_id = self.vram[map_base + map_index];
            let tile_index = self.resolve_bg_tile_index(tile_id);
            *out = self.tile_data[tile_index][pixel_row * 8 + pixel_col];
        }

        line
    }

    fn render_bg_scanline(&mut self, y: usize, bg_color_ids: &[u8; SCREEN_WIDTH]) {
        if y >= SCREEN_HEIGHT {
            return;
        }

        for (x, color_id) in bg_color_ids.iter().enumerate() {
            self.screen[y][x] = self.map_palette_color(self.bgp, *color_id);
        }
    }

    fn render_window_scanline(&mut self, y: usize, bg_color_ids: &mut [u8; SCREEN_WIDTH]) {
        if y >= SCREEN_HEIGHT || !self.window_enabled() || (y as u8) < self.wy {
            return;
        }

        let map_base = self.window_map_base_offset();
        let window_origin_x = self.wx as i16 - 7;
        let window_y = (y as u8).wrapping_sub(self.wy) as usize;
        let tile_row = window_y / 8;
        let pixel_row = window_y % 8;

        for x in 0..SCREEN_WIDTH {
            if (x as i16) < window_origin_x {
                continue;
            }

            let window_x = (x as i16 - window_origin_x) as usize;
            let tile_col = window_x / 8;
            let pixel_col = window_x % 8;

            let map_index = tile_row * 32 + tile_col;
            let tile_id = self.vram[map_base + map_index];
            let tile_index = self.resolve_bg_tile_index(tile_id);
            let color_id = self.tile_data[tile_index][pixel_row * 8 + pixel_col];

            bg_color_ids[x] = color_id;
            self.screen[y][x] = self.map_palette_color(self.bgp, color_id);
        }
    }

    fn render_obj_scanline(&mut self, y: usize, bg_color_ids: &[u8; SCREEN_WIDTH]) {
        if y >= SCREEN_HEIGHT || !self.obj_enabled() {
            return;
        }

        let sprite_height = self.obj_height() as i16;
        let mut visible_sprites: Vec<(i16, i16, u8, u8, usize)> = Vec::with_capacity(10);

        for index in 0..40 {
            let base = index * 4;
            let sprite_y = self.oam[base] as i16 - 16;
            let sprite_x = self.oam[base + 1] as i16 - 8;
            let tile_index = self.oam[base + 2];
            let attrs = self.oam[base + 3];

            let line = y as i16;
            if line >= sprite_y && line < sprite_y + sprite_height {
                visible_sprites.push((sprite_x, sprite_y, tile_index, attrs, index));
                if visible_sprites.len() == 10 {
                    break;
                }
            }
        }

        for x in 0..SCREEN_WIDTH {
            let mut best_pixel: Option<(u8, i16, usize)> = None;

            for (sprite_x, sprite_y, tile_id, attrs, index) in &visible_sprites {
                let sx = *sprite_x;
                if (x as i16) < sx || (x as i16) >= sx + 8 {
                    continue;
                }

                let mut row = y as i16 - *sprite_y;
                let mut col = x as i16 - sx;

                if (attrs & 0x40) != 0 {
                    row = sprite_height - 1 - row;
                }
                if (attrs & 0x20) != 0 {
                    col = 7 - col;
                }

                let tile_index = if sprite_height == 16 {
                    let base_tile = (*tile_id & 0xFE) as usize;
                    if row >= 8 {
                        base_tile + 1
                    } else {
                        base_tile
                    }
                } else {
                    *tile_id as usize
                };

                let tile_row = (row as usize) % 8;
                let tile_col = col as usize;
                let color_id = self.tile_data[tile_index][tile_row * 8 + tile_col];
                if color_id == 0 {
                    continue;
                }

                let behind_bg = (attrs & 0x80) != 0;
                if behind_bg && bg_color_ids[x] != 0 {
                    continue;
                }

                let palette = if (attrs & 0x10) != 0 { self.obp1 } else { self.obp0 };
                let mapped = self.map_palette_color(palette, color_id);
                let priority_key = (sx, *index);

                if let Some((_, best_x, best_idx)) = best_pixel {
                    if priority_key < (best_x, best_idx) {
                        best_pixel = Some((mapped, sx, *index));
                    }
                } else {
                    best_pixel = Some((mapped, sx, *index));
                }
            }

            if let Some((pixel, _, _)) = best_pixel {
                self.screen[y][x] = pixel;
            }
        }
    }

    fn bg_enabled(&self) -> bool {
        (self.lcdc & 0x01) != 0
    }

    fn bg_map_base_offset(&self) -> usize {
        if (self.lcdc & 0x08) != 0 {
            BG_MAP1_OFFSET
        } else {
            BG_MAP0_OFFSET
        }
    }

    fn window_enabled(&self) -> bool {
        (self.lcdc & 0x20) != 0
    }

    fn window_map_base_offset(&self) -> usize {
        if (self.lcdc & 0x40) != 0 {
            BG_MAP1_OFFSET
        } else {
            BG_MAP0_OFFSET
        }
    }

    fn obj_enabled(&self) -> bool {
        (self.lcdc & 0x02) != 0
    }

    fn obj_height(&self) -> u8 {
        if (self.lcdc & 0x04) != 0 {
            16
        } else {
            8
        }
    }

    fn resolve_bg_tile_index(&self, tile_id: u8) -> usize {
        // LCDC bit4:
        // 1 => unsigned tile IDs, base 0x8000 (tile 0..255)
        // 0 => signed tile IDs, base 0x9000 (ID interpreted as i8)
        if (self.lcdc & 0x10) != 0 {
            tile_id as usize
        } else {
            let signed = tile_id as i8 as i16;
            (256i16 + signed) as usize
        }
    }

    fn map_palette_color(&self, palette: u8, color_id: u8) -> u8 {
        let shift = (color_id & 0x03) * 2;
        (palette >> shift) & 0x03
    }
}
