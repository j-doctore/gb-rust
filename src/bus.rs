use crate::cartridge::Cartridge;
use crate::ppu::Ppu;
const WRAM_SIZE: usize = 1024 * 8; //8KiB
const IO_SIZE: usize = 0x80; // FF00-FF7F (128 bytes)
const HRAM_SIZE: usize = 0x7F; // FF80-FFFE (127 bytes)

pub struct MemoryBus {
    cartridge: Cartridge,
    ppu: Ppu,

    wram: [u8; WRAM_SIZE],
    hram: [u8; HRAM_SIZE],
    io: [u8; IO_SIZE],
}

impl MemoryBus {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cartridge,
            ppu: Ppu::new(),
            wram: [0; WRAM_SIZE],
            hram: [0; HRAM_SIZE],
            io: [0; IO_SIZE],
        }
    }

    pub fn ppu(&self) -> &Ppu {
        &self.ppu
    }

    pub fn ppu_mut(&mut self) -> &mut Ppu {
        &mut self.ppu
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            //16KiB ROM Bank
            0x0000..=0x3FFF => self.cartridge.rom[addr as usize],
            //16KiB ROM Bank
            0x4000..=0x7FFF => self.cartridge.rom[addr as usize],
            //VRAM
            0x8000..=0x9FFF => self.ppu.read_vram(addr as usize - 0x8000),
            //8KiB External RAM?
            0xA000..=0xBFFF => {
                let ram_addr = addr as usize - 0xA000;
                if ram_addr < self.cartridge.ram.len() {
                    self.cartridge.ram[ram_addr]
                } else {
                    0xFF
                }
            }
            //8KiB WRAM
            0xC000..=0xDFFF => self.wram[addr as usize - 0xC000],
            //Echo RAM 0xE000..=0xFDFF
            0xE000..=0xFDFF => self.wram[addr as usize - 0x2000],
            //OAM
            0xFE00..=0xFE9F => self.ppu.read_oam(addr as usize - 0xFE00),
            //IO Registers
            0xFF40..=0xFF4B => self.ppu.read_reg(addr),

            0xFF00..=0xFF7F => self.io[addr as usize - 0xFF00],

            //HRAM
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80],
            //Interrupt
            0xFFFF => 0,
            //0xFEA0..=0xFEFF prohibited
            _ => 0xFF,
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            //==ROM Bank== IGNORE
            0x0000..=0x7FFF => (),
            //VRAM
            0x8000..=0x9FFF => self.ppu.write_vram(addr as usize - 0x8000, value),
            //8KiB External RAM?
            0xA000..=0xBFFF => {
                let ram_addr = addr as usize - 0xA000;
                if ram_addr < self.cartridge.ram.len() {
                    self.cartridge.ram[ram_addr] = value;
                }
            }
            //8KiB WRAM
            0xC000..=0xDFFF => self.wram[addr as usize - 0xC000] = value,
            //Echo RAM 0xE000..=0xFDFF
            0xE000..=0xFDFF => self.wram[addr as usize - 0x2000] = value,
            //OAM
            0xFE00..=0xFE9F => self.ppu.write_oam(addr as usize - 0xFE00, value),
            //IO Registers
            0xFF40..=0xFF4B => self.ppu.write_reg(addr, value),
            0xFF00..=0xFF7F => {
                self.io[addr as usize - 0xFF00] = value;
                //DEBUG BLARGG:
                if addr == 0xFF02 && value == 0x81 {
                    let c = self.io[0x01] as char;
                    println!("{}", c);
                }
            }
            //HRAM
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80] = value,
            //Interrupt
            0xFFFF => (),
            //0xFEA0..=0xFEFF prohibited ?> IGNORE
            _ => (),
        }
    }
}
