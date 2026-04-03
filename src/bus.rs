use crate::cartridge::Cartridge;
use crate::interrupts::InterruptType;
use crate::ppu::Ppu;
use crate::timer::TimerRegister;
use crate::joypad::Joypad;
const WRAM_SIZE: usize = 1024 * 8; //8KiB
const IO_SIZE: usize = 0x80; // FF00-FF7F (128 bytes)
const HRAM_SIZE: usize = 0x7F; // FF80-FFFE (127 bytes)

pub struct MemoryBus {
    cartridge: Cartridge,
    ppu: Ppu,
    timers: TimerRegister,
    joypad: Joypad,
    wram: [u8; WRAM_SIZE],
    hram: [u8; HRAM_SIZE],
    io: [u8; IO_SIZE],
    if_reg: u8,
    ie: u8,
}

impl MemoryBus {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cartridge,
            ppu: Ppu::new(),
            timers: TimerRegister::new(),
            joypad: Joypad::new(),
            wram: [0; WRAM_SIZE],
            hram: [0; HRAM_SIZE],
            io: [0; IO_SIZE],
            if_reg: 0,
            ie: 0,
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
            0xE000..=0xFDFF => self.wram[addr as usize - 0xE000],
            //OAM
            0xFE00..=0xFE9F => self.ppu.read_oam(addr as usize - 0xFE00),
            //PPU Registers
            0xFF40..=0xFF4B => self.ppu.read_reg(addr),
            //IO Registers
            0xFF00..=0xFF7F => match addr & 0x00FF {
                0x00 => self.joypad.read(),
                0x0F => self.if_reg | 0xE0,
                0x04..=0x07 => self.timers.read_byte(addr),
                //TODO
                _ => self.io[addr as usize - 0xFF00],
            },

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
            0xE000..=0xFDFF => self.wram[addr as usize - 0xE000] = value,
            //OAM
            0xFE00..=0xFE9F => self.ppu.write_oam(addr as usize - 0xFE00, value),
            //IO Registers
            0xFF40..=0xFF4B => {
                self.ppu.write_reg(addr, value);
                if addr == 0xFF46 {
                    self.do_oam_dma(value);
                }
            }
            0xFF00..=0xFF7F => {
                match addr & 0xFF {
                    0x00 => self.joypad.write(value),
                    0x0F => self.if_reg = value & 0x1F,
                    0x04..=0x07 => self.timers.write_byte(addr, value),
                    //TODO
                    _ => {
                        self.io[addr as usize - 0xFF00] = value;
                        //DEBUG BLARGG:
                        if addr == 0xFF02 && value == 0x81 {
                            let c = self.io[0x01] as char;
                            print!("{}", c);
                        }
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

    //TODO: improve and make this consistent. Bus doesnt use it everywhere
    fn do_oam_dma(&mut self, page: u8) {
        let base = (page as u16) << 8;
        for i in 0..0xA0u16 {
            let src = base.wrapping_add(i);
            let value = self.read_byte(src);
            self.ppu.dma_write_oam(i as usize, value);
        }
    }

    pub fn tick(&mut self, cycles: u32) {
        let (entered_vblank, entered_stat_irq) = self.ppu.step(cycles);
        if entered_vblank {
            self.request_interrupt(InterruptType::VBlank);
        }
        if entered_stat_irq {
            self.request_interrupt(InterruptType::LCDSTAT);
        }
        if self.timers.step(cycles) {
            self.request_interrupt(InterruptType::Timer);
        }
    }

    pub fn pending_interrupts(&self) -> u8 {
        (self.ie & self.if_reg) & 0x1F
    }

    pub fn acknowledge_interrupt(&mut self, irq_bit: u8) {
        self.if_reg &= !(1 << irq_bit);
    }

    pub fn request_interrupt(&mut self, irq: InterruptType) {
        let bit: usize = irq.into();
        self.if_reg |= 1 << bit;
    }
}
