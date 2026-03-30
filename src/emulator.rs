use crate::cpu::Cpu;
use crate::register::Register;

const RAM_SIZE: usize = 1024 * 8; //8KiB
const VRAM_SIZE: usize = 1024 * 8; //8KiB

pub struct Emulator {
    registers: Register,
    ram: [u8; RAM_SIZE],
    vram: [u8; VRAM_SIZE],
    screen: [[u8; 160]; 144],
}

impl Emulator {
    pub fn read(&self, addr: u16) {
        match addr {
            //16KiB ROM Bank
            0x0000..=0x3FFF => {}
            //16KiB ROM Bank
            0x4000..=0x7FFF => {}
            //VRAM
            0x8000..=0x9FFF => {}
            //8KiB External RAM?
            0xA000..=0xBFFF => {}
            //4KiB WRAM
            0xC000..=0xCFFF => {}
            //4KiB WRAM
            0xD000..=0xDFFF => {}
            //Echo RAM (DO NOT USE)
            0xE000..=0xFDFF => {}
            //OAM
            0xFE00..=0xFE9F => {}
            //(DO NOT USE)
            0xFEA0..=0xFEFF => {}
            //IO Registers
            0xFF00..=0xFF7F => {}
            //HRAM
            0xFF80..=0xFFFE => {}
            //Interrupt
            0xFFFF..=0xFFFF => {}
        }
    }

    pub fn new() -> Self {
        Emulator {
            registers: Register::new(),
            ram: [0; RAM_SIZE],
            vram: [0; RAM_SIZE],
            screen: [[0; 160]; 144],
        }
    }

    pub fn load_rom(&self, rom: &String) {
        todo!()
    }
}
