use crate::bus::MemoryBus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;

pub struct Emulator {
    cpu: Cpu,
    bus: MemoryBus,
}

impl Emulator {
    pub fn get_display(&self) -> &[[u8; 160]; 144] {
        &self.bus.ppu().get_display()
    }

    pub fn new(rom_path: &str) -> Self {
        Emulator {
            cpu: Cpu::new(),
            bus: MemoryBus::new(Cartridge::new(rom_path)),
        }
    }

    pub fn tick(&mut self) {
        let cycles = self.cpu.step(&mut self.bus);
        //self.bus.ppu_mut().step(cycles);
    }
}
