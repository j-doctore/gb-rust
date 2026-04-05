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
        self.bus.ppu().get_display()
    }

    pub fn serial_output(&self) -> &str {
        self.bus.serial_output()
    }

    pub fn debug_cpu_pc(&self) -> u16 {
        self.cpu.pc()
    }

    pub fn debug_peek_byte(&self, addr: u16) -> u8 {
        self.bus.read_byte(addr)
    }

    pub fn debug_cpu_a(&self) -> u8 {
        self.cpu.a()
    }

    pub fn debug_last_ext_ram_write(&self) -> Option<(u16, u8)> {
        self.bus.last_ext_ram_write()
    }

    pub fn new(rom_path: &str) -> Result<Self, String> {
        let cart = Cartridge::new(rom_path)?;
        Ok(Emulator { cpu: Cpu::new(), bus: MemoryBus::new(cart) })
    }

    pub fn tick(&mut self) -> u32 {
        let cycles = self.cpu.step(&mut self.bus);
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
