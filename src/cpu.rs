use crate::bus::MemoryBus;
use crate::register::Register;

pub struct Cpu {
	registers: Register,
	pc: u16,
	sp: u16,
}

impl Cpu {
	pub fn new() -> Self {
		Self {
			registers: Register::new(),
			pc: 0x0100,
			sp: 0xFFFE,
		}
	}

	fn fetch_byte(&mut self, bus: &MemoryBus) -> u8 {
		let byte = bus.read_byte(self.pc);
		self.pc += 1;
		byte
	}

	fn push(&mut self, bus: &mut MemoryBus, value: u8) {
		bus.write_byte(self.sp, value);
		self.sp -= 1;
	}

	fn pop(&mut self, bus: &MemoryBus) -> u8 {
		self.sp += 1;
		bus.read_byte(self.sp)
	}

	pub fn step(&mut self, bus: &mut MemoryBus) {
		let opcode = self.fetch_byte(bus);

		match opcode {
			0x00 => {}
			_ => todo!("Opcode {:02X} not implemented", opcode),
		}
	}
}