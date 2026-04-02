use crate::bus::{self, MemoryBus};
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

    pub fn read_r(&self, bus: &MemoryBus, value: u8) -> u8 {
        match value {
            0 => self.registers.get_b(),
            1 => self.registers.get_c(),
            2 => self.registers.get_d(),
            3 => self.registers.get_e(),
            4 => self.registers.get_h(),
            5 => self.registers.get_l(),
            6 => bus.read_byte(self.registers.get_hl()),
            7 => self.registers.get_a(),
            _ => panic!("Invalid register: {}", value),
        }
    }

    pub fn write_r(&mut self, bus: &mut MemoryBus, value: u8, data: u8) {
        match value {
            0 => self.registers.set_b(data),
            1 => self.registers.set_c(data),
            2 => self.registers.set_d(data),
            3 => self.registers.set_e(data),
            4 => self.registers.set_h(data),
            5 => self.registers.set_l(data),
            6 => bus.write_byte(self.registers.get_hl(), data),
            7 => self.registers.set_a(data),
            _ => panic!("Invalid register: {}", value),
        }
    }

    pub fn step(&mut self, bus: &mut MemoryBus) {
        let opcode = self.fetch_byte(bus);

        //XXYYYZZZ
        match opcode {
            0x00 => {},
            //LOAD
            0x40..=0x7F => {
                if (opcode == 0x76) {
                    todo!("HALT instruction not implemented");
                    // HALT
                } else {
                    let y = (opcode & 0b0011_1000) >> 3;
                    let z = opcode & 0b0000_0111;
                    let value = self.read_r(bus, z);
                    self.write_r(bus, y, value);
                }
            }
			//ALU
			0x80..=0xBF => {
				let y = (opcode & 0b0011_1000) >> 3;
				let z = opcode & 0b0000_0111;
				self.alu(y, z);
			}
            _ => todo!("Opcode {:02X} not implemented", opcode),
        }
    }

	fn alu(&mut self, y: u8, z: u8) {
		todo!()
	}
}
