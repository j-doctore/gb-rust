use crate::bus::MemoryBus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;

pub struct Emulator {
    cpu: Cpu,
    bus: MemoryBus,

    //Display
    display: [[u8; 160]; 144],
}

impl Emulator {
    pub fn get_display(&self) -> &[[u8; 160]; 144] {
        &self.display
    }

    pub fn new(rom_path: &str) -> Self {
        Emulator {
            display: [[0; 160]; 144],
            cpu: Cpu::new(),
            bus: MemoryBus::new(Cartridge::new(rom_path)),
        }
    }

    pub fn tick(&mut self) {
        self.cpu.step(&mut self.bus);
    }
}
