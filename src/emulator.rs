use crate::membus::MemoryBus;
use crate::cpu::Cpu;
use crate::joypad::UserInput;
use crate::Cartridge;

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
        let cycles = self.cpu.step(&mut self.bus);
        self.bus.step(cycles);
        cycles
    }

    pub fn press_input(&mut self, input: UserInput) {
        self.bus.press_input(input);
    }

    pub fn release_input(&mut self, input: UserInput) {
        self.bus.release_input(input);
    }

    pub fn run_cycles(&mut self, cycles: u32) {
        let mut elapsed = 0u32;
        while elapsed < cycles {
            elapsed = elapsed.saturating_add(self.tick());
        }
    }
}
