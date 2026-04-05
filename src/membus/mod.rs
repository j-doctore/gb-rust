use crate::io::IoRegisters;
use crate::interrupts::InterruptType;
use crate::joypad::UserInput;

mod cartridge;
pub use self::cartridge::Cartridge;
use crate::ppu::Ppu;
const WRAM_SIZE: usize = 1024 * 8; //8KiB
const HRAM_SIZE: usize = 0x7F; // FF80-FFFE (127 bytes)

pub struct MemoryBus {
    cartridge: Cartridge,
    ppu: Ppu,
    io: IoRegisters,

    wram: [u8; WRAM_SIZE],
    hram: [u8; HRAM_SIZE],
    ie: u8,
    last_ext_ram_write: Option<(u16, u8)>,
}

impl MemoryBus {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cartridge,
            ppu: Ppu::new(),
            wram: [0; WRAM_SIZE],
            hram: [0; HRAM_SIZE],
            io: IoRegisters::new(),
            ie: 0,
            last_ext_ram_write: None,
        }
    }

    pub fn ppu(&self) -> &Ppu {
        &self.ppu
    }

    pub fn step(&mut self, cycles: u32) {
        self.io.step(cycles);

        let (entered_vblank, entered_stat_irq) = self.ppu.step(cycles);
        if entered_vblank {
            self.io.request_interrupt(InterruptType::VBlank);
        }
        if entered_stat_irq {
            self.io.request_interrupt(InterruptType::LcdStat);
        }
    }

    pub fn serial_output(&self) -> &str {
        self.io.serial_output()
    }

    pub fn last_ext_ram_write(&self) -> Option<(u16, u8)> {
        self.last_ext_ram_write
    }

    pub fn press_input(&mut self, input: UserInput) {
        self.io.press_input(input);
    }

    pub fn release_input(&mut self, input: UserInput) {
        self.io.release_input(input);
    }

    //TODO; BANKING, OAM DMA
    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            //16KiB ROM Bank
            0x0000..=0x3FFF => self.cartridge.read_rom(addr),
            //16KiB ROM Bank
            0x4000..=0x7FFF => self.cartridge.read_rom(addr),
            //VRAM
            0x8000..=0x9FFF => self.ppu.read_vram(addr as usize),
            //8KiB External RAM?
            0xA000..=0xBFFF => self.cartridge.read_ext_ram(addr),
            //8KiB WRAM
            0xC000..=0xDFFF => self.wram[addr as usize - 0xC000],
            //Echo RAM 0xE000..=0xFDFF
            0xE000..=0xFDFF => self.wram[addr as usize - 0xE000],
            //OAM
            0xFE00..=0xFE9F => self.ppu.read_oam(addr as usize - 0xFE00),

            //IO Registers
            0xFF00..=0xFF7F => {
                match addr & 0x00FF {
                    //
                    0x00..=0x3F => self.io.read_io_reg(addr),
                    //PPU Registers
                    0x40..=0x4B => self.ppu.read_reg(addr),
                    _ => self.io.read_io_reg(addr),
                }
            }
            //HRAM
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80],
            //Interrupt
            0xFFFF => self.ie,
            //0xFEA0..=0xFEFF prohibited
            _ => 0xFF,
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            //==ROM Bank== IGNORE
            0x0000..=0x7FFF => (),
            //VRAM
            0x8000..=0x9FFF => self.ppu.write_vram(addr as usize, value),
            //8KiB External RAM?
            0xA000..=0xBFFF => {
                self.last_ext_ram_write = Some((addr, value));
                self.cartridge.write_ram(addr, value)
            }
            //8KiB WRAM
            0xC000..=0xDFFF => self.wram[addr as usize - 0xC000] = value,
            //Echo RAM 0xE000..=0xFDFF
            0xE000..=0xFDFF => self.wram[addr as usize - 0xE000] = value,
            //OAM
            0xFE00..=0xFE9F => self.ppu.write_oam(addr as usize - 0xFE00, value),

            //IO Registers
            0xFF00..=0xFF7F => {
                match addr & 0x00FF {
                    //
                    0x00..=0x3F => self.io.write_io(addr, value),
                    //PPU Registers
                    0x40..=0x4B => self.ppu.write_reg(addr, value),
                    _ => self.io.write_io(addr, value),
                }

                if addr == 0xFF46 {
                    // OAM DMA: source = value << 8, copy 160 bytes to FE00..FE9F
                    let source_base = (value as u16) << 8;
                    for i in 0..0xA0u16 {
                        let byte = self.read_byte(source_base.wrapping_add(i));
                        self.ppu.dma_write_oam(i as usize, byte);
                    }
                }
            }

            //HRAM
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80] = value,
            //Interrupt
            0xFFFF => self.ie = value,
            //0xFEA0..=0xFEFF prohibited => IGNORE
            _ => (),
        }
    }
}