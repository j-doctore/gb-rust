const VRAM_SIZE: usize = 1024 * 8; //8KiB
const OAM_SIZE: usize = 0xA0; // FE00-FE9F (160 bytes)

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
            0xFF40 => self.lcdc,
            0xFF41 => {
                // Lower 2 bits = mode, bit2 = coincidence. You can compose here:
                let mode_bits = (self.mode as u8) & 0b11;
                let coincidence_bit = if self.ly == self.lyc { 0b100 } else { 0 };
                mode_bits | coincidence_bit
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

    pub fn step(&mut self, cycles: u32) {
        todo!();
    }

}
