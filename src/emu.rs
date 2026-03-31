use crate::register::Register;

const RAM_SIZE: usize = 1024 * 8; //8KiB
const VRAM_SIZE: usize = 1024 * 8; //8KiB

pub struct Emulator {
    //Registers
    registers: Register,
    //Pointers
    pc: u16,
    sp: u16,
    //RAM
    wram: [u8; RAM_SIZE],
    vram: [u8; VRAM_SIZE],

    screen: [[u8; 160]; 144],

}

impl Emulator {
    pub fn get_screen(&self) -> &[[u8; 160]; 144] {
        &self.screen
    }

    //TODO: proper reading
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            //16KiB ROM Bank
            0x0000..=0x3FFF => {0}
            //16KiB ROM Bank
            0x4000..=0x7FFF => {0}
            //VRAM
            0x8000..=0x9FFF => {self.vram[addr as usize - 0x8000] }
            //8KiB External RAM?
            0xA000..=0xBFFF => {0}
            //4KiB WRAM
            0xC000..=0xCFFF => {self.wram[addr as usize - 0xC000] }
            //4KiB WRAM
            0xD000..=0xDFFF => {self.wram[addr as usize - 0xD000] }
            //Echo RAM (DO NOT USE)
            0xE000..=0xFDFF => {0}
            //OAM
            0xFE00..=0xFE9F => {0}
            //(DO NOT USE)
            0xFEA0..=0xFEFF => {0}
            //IO Registers
            0xFF00..=0xFF7F => {0}
            //HRAM
            0xFF80..=0xFFFE => {0}
            //Interrupt
            0xFFFF..=0xFFFF => {0}
        }
    }

    pub fn new() -> Self {
        Emulator {
            pc: 0x0100,
            sp: 0xFFFE,
            registers: Register::new(),
            wram: [0; RAM_SIZE],
            vram: [0; RAM_SIZE],
            screen: [[0; 160]; 144],
        }
    }

    pub fn load_rom(&self, rom: &String) {
        todo!()
    }
}
