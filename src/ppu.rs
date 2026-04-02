const VRAM_SIZE: usize = 1024 * 8; //8KiB
const OAM_SIZE: usize = 0xA0; // FE00-FE9F (160 bytes)

const FF40_LCDC: u16 = 0xFF40;
const FF41_STAT: u16 = 0xFF41;
const FF42_SCY: u16 = 0xFF42;
const FF43_SCX: u16 = 0xFF43;
const FF44_LY: u16 = 0xFF44;
const FF45_LYC: u16 = 0xFF45;
const FF46_DMA: u16 = 0xFF46;
const FF47_BGP: u16 = 0xFF47;
const FF48_OBP0: u16 = 0xFF48;
const FF49_OBP1: u16 = 0xFF49;
const FF4A_WY: u16 = 0xFF4A;
const FF4B_WX: u16 = 0xFF4B;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Transfer = 3,
}

pub struct Ppu {
    screen: [[u8; 160]; 144],

    oam: [u8; OAM_SIZE],
    vram: [u8; VRAM_SIZE],

    lcdc: u8,  // FF40
    stat: u8,  // FF41 (upper bits writable, lower bits mostly status)
    scy:  u8,  // FF42
    scx:  u8,  // FF43
    ly:   u8,  // FF44 (read-only from CPU POV)
    lyc:  u8,  // FF45
    bgp:  u8,  // FF47
    obp0: u8,  // FF48
    obp1: u8,  // FF49
    wy:   u8,  // FF4A
    wx:   u8,  // FF4B

    mode: Mode,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            screen: [[0; 160]; 144],

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
        }
    }

    pub fn get_display(&self) -> &[[u8; 160]; 144] {
        &self.screen
    }

    pub fn get_ly(&self) -> u8 {
        self.ly
    }
    pub fn set_ly(&mut self, value: u8) {
        self.ly = value;
    }

    pub fn get_lyc(&self) -> u8 {
        self.lyc
    }
    pub fn set_lyc(&mut self, value: u8) {
        self.lyc = value;
    }

    //Todo: mode logic
    pub fn read_vram(&self, addr: usize) -> u8 {
        self.vram[addr]
    }

    pub fn write_vram(&mut self, addr: usize, value: u8) {
        self.vram[addr] = value;
    }

    pub fn read_oam(&self, addr: usize) -> u8 {
        self.oam[addr]
    }

    pub fn write_oam(&mut self, addr: usize, value: u8) {
        self.oam[addr] = value;
    }

    pub fn read_reg(&self, addr: u16) -> u8 {
        match addr {
            FF40_LCDC => self.lcdc,
            FF41_STAT => {
                // Lower 2 bits = mode, bit2 = coincidence. You can compose here:
                // Keep it simple early: reflect mode in bits0-1.
                let mode_bits = (self.mode as u8) & 0b11;
                (self.stat & !0b11) | mode_bits
            }
            FF42_SCY => self.scy,
            FF43_SCX => self.scx,
            FF44_LY  => self.ly,
            FF45_LYC => self.lyc,
            FF46_DMA => 0xFF, // often reads as last written / undefined; safe: 0xFF
            FF47_BGP => self.bgp,
            FF48_OBP0 => self.obp0,
            FF49_OBP1 => self.obp1,
            FF4A_WY => self.wy,
            FF4B_WX => self.wx,
            _ => 0xFF,
        }
    }

    pub fn write_reg(&mut self, addr: u16, value: u8) {
        match addr {
            FF40_LCDC => self.lcdc = value,
            FF41_STAT => {
                // Bits 0-2 are status (mode + coincidence), typically read-only.
                // Bits 3-6 are interrupt enables (writable).
                // Early mask: keep lower 3 bits, allow 3..6 from value.
                self.stat = (self.stat & 0b0000_0111) | (value & 0b0111_1000);
            }
            FF42_SCY => self.scy = value,
            FF43_SCX => self.scx = value,
            FF44_LY  => {
                // Usually read-only; many emus ignore writes.
                // Keep ignore for now:
                let _ = value;
            }
            FF45_LYC => self.lyc = value,
            FF46_DMA => {
                // Latch request; actual copy should be performed by Bus/Emu after the write finishes
                //self.dma_requested = Some(value);
            }
            FF47_BGP => self.bgp = value,
            FF48_OBP0 => self.obp0 = value,
            FF49_OBP1 => self.obp1 = value,
            FF4A_WY => self.wy = value,
            FF4B_WX => self.wx = value,
            _ => {}
        }
    }
}
