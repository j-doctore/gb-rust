use crate::cartridge::{self, Cartridge};
use crate::register::Register;

const WRAM_SIZE: usize = 1024 * 8; //8KiB
const VRAM_SIZE: usize = 1024 * 8; //8KiB
const OAM_SIZE: usize = 0xA0;  // FE00-FE9F (160 bytes)
const IO_SIZE: usize = 0x80;   // FF00-FF7F (128 bytes)
const HRAM_SIZE: usize = 0x7F; // FF80-FFFE (127 bytes)

pub struct Emulator {
    cartridge: Cartridge,
    //Registers
    registers: Register,
    //Pointers
    pc: u16,
    sp: u16,
    //RAM
    wram: [u8; WRAM_SIZE],
    vram: [u8; VRAM_SIZE],
    hram: [u8; HRAM_SIZE],
    oam: [u8; OAM_SIZE],

    io: [u8; IO_SIZE],

    //Display
    display: [[u8; 160]; 144],
}

impl Emulator {
    pub fn get_display(&self) -> &[[u8; 160]; 144] {
        &self.display
    }

    //TODO: proper ROM Banking, Interrupt
    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            //16KiB ROM Bank
            0x0000..=0x3FFF => self.cartridge.rom[addr as usize],
            //16KiB ROM Bank
            0x4000..=0x7FFF => self.cartridge.rom[addr as usize],
            //VRAM
            0x8000..=0x9FFF => self.vram[addr as usize - 0x8000],
            //8KiB External RAM?
            0xA000..=0xBFFF => {
                let ram_addr = addr as usize - 0xA000;
                if ram_addr < self.cartridge.ram.len() {
                    self.cartridge.ram[ram_addr]
                } else {
                    0xFF
                }
            },
            //8KiB WRAM
            0xC000..=0xDFFF => self.wram[addr as usize - 0xC000],
            //OAM
            0xFE00..=0xFE9F => self.oam[addr as usize - 0xFE00],
            //IO Registers
            0xFF00..=0xFF7F => self.io[addr as usize - 0xFF00],
            //HRAM
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80],
            //Interrupt
            0xFFFF => 0,
            //Echo RAM 0xE000..=0xFDFF and 0xFEA0..=0xFEFF prohibited
            _ => 0xFF,
        }
    }

    //TODO: proper writing, Interrupt
    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            //==ROM Bank== IGNORE
            0x0000..=0x7FFF => (),
            //VRAM
            0x8000..=0x9FFF => self.vram[addr as usize - 0x8000] = value,
            //8KiB External RAM?
            0xA000..=0xBFFF => {
                let ram_addr = addr as usize - 0xA000;
                if ram_addr < self.cartridge.ram.len() {
                    self.cartridge.ram[ram_addr] = value;
                }
            },
            //8KiB WRAM
            0xC000..=0xDFFF => self.wram[addr as usize - 0xC000] = value,
            //OAM
            0xFE00..=0xFE9F => self.oam[addr as usize - 0xFE00] = value,
            //IO Registers
            0xFF00..=0xFF7F => self.io[addr as usize - 0xFF00] = value,
            //HRAM
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80] = value,
            //Interrupt
            0xFFFF => (),
            //Echo RAM 0xE000..=0xFDFF and 0xFEA0..=0xFEFF prohibited ?> IGNORE
            _ => (),
        }
    }

    pub fn new(rom_path: &str) -> Self {
        Emulator {
            pc: 0x0100,
            sp: 0xFFFE,
            registers: Register::new(),
            wram: [0; WRAM_SIZE],
            vram: [0; WRAM_SIZE],
            hram: [0; HRAM_SIZE],
            oam: [0; OAM_SIZE],
            io: [0; IO_SIZE],

            display: [[0; 160]; 144],

            cartridge: Cartridge::new(rom_path),
        }
    }

    fn push(&mut self, value: u8) {
        self.write(self.sp, value);
        self.sp -= 1;
    }

    fn pop(&mut self) -> u8 {
        self.sp += 1;
        self.read_byte(self.sp)
    }

    fn fetch(&mut self) -> u8 {
        todo!()
    }

    pub fn step(&mut self) {
        //fetch opcode
        let opcode = self.fetch();
        todo!();
        //execute opcode
        match opcode {
            0x00 => (), //NOP
            _ => todo!("Opcode {:02X} not implemented", opcode),
        }
    }
}
