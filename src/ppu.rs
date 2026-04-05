const VRAM_SIZE: usize = 1024 * 8; //8KiB
const OAM_SIZE: usize = 0xA0; // FE00-FE9F (160 bytes)

const SCREEN_HEIGHT: usize = 144;
const SCREEN_WIDTH: usize = 160;

const TILE_SIZE: usize = 8 * 8;

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
        if addr < 0x8000 || addr > 0x97FF {
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

    fn calc_bottom_right(&self) -> (usize, usize) {
        let bottom = (self.scy + 143) as usize % 256;
        let right = (self.scx + 159) as usize % 256;
        (bottom, right)
    }
}
