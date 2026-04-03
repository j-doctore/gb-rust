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

    pub fn new(rom_path: &str) -> Result<Self, String> {
        let cart = Cartridge::new(rom_path)?;
        Ok(Emulator { cpu: Cpu::new(), bus: MemoryBus::new(cart) })
    }

    pub fn tick(&mut self) -> u32 {
        self.cpu.step(&mut self.bus)
    }

    pub fn run_cycles(&mut self, cycles: u32) {
        let mut elapsed = 0u32;
        while elapsed < cycles {
            elapsed = elapsed.saturating_add(self.tick());
        }
    }
}
