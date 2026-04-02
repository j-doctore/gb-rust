
const VRAM_SIZE: usize = 1024 * 8; //8KiB
const OAM_SIZE: usize = 0xA0; // FE00-FE9F (160 bytes)

pub struct Ppu {
    oam: [u8; OAM_SIZE],
    vram: [u8; VRAM_SIZE],
    
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            oam: [0; OAM_SIZE],
            vram: [0; VRAM_SIZE],
        }
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
}
