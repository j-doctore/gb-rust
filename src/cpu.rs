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

    fn push_u8(&mut self, bus: &mut MemoryBus, value: u8) {
        self.sp -= 1;
        bus.write_byte(self.sp, value);
    }

    fn pop_u8(&mut self, bus: &MemoryBus) -> u8 {
        let value = bus.read_byte(self.sp);
        self.sp += 1;
        value
    }

    fn push_u16(&mut self, bus: &mut MemoryBus, value: u16) {
        let hi = (value >> 8) as u8;
        let lo = value as u8;
        self.push_u8(bus, hi);
        self.push_u8(bus, lo);
    }

    fn pop_u16(&mut self, bus: &MemoryBus) -> u16 {
        let lo = self.pop_u8(bus) as u16;
        let hi = self.pop_u8(bus) as u16;
        (hi << 8) | lo
    }

    fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        self.registers.set_flag_z(z);
        self.registers.set_flag_n(n);
        self.registers.set_flag_h(h);
        self.registers.set_flag_c(c);
    }

    fn imm8(&mut self, bus: &MemoryBus) -> u8 {
        self.fetch_byte(bus)
    }

    fn imm16(&mut self, bus: &MemoryBus) -> u16 {
        let lo = self.fetch_byte(bus) as u16;
        let hi = self.fetch_byte(bus) as u16;
        (hi << 8) | lo
    }

    pub fn read_r(&self, bus: &MemoryBus, r_index: u8) -> u8 {
        match r_index {
            0 => self.registers.get_b(),
            1 => self.registers.get_c(),
            2 => self.registers.get_d(),
            3 => self.registers.get_e(),
            4 => self.registers.get_h(),
            5 => self.registers.get_l(),
            6 => bus.read_byte(self.registers.get_hl()),
            7 => self.registers.get_a(),
            _ => panic!("Invalid register: {}", r_index),
        }
    }

    pub fn write_r(&mut self, bus: &mut MemoryBus, r_index: u8, data: u8) {
        match r_index {
            0 => self.registers.set_b(data),
            1 => self.registers.set_c(data),
            2 => self.registers.set_d(data),
            3 => self.registers.set_e(data),
            4 => self.registers.set_h(data),
            5 => self.registers.set_l(data),
            6 => bus.write_byte(self.registers.get_hl(), data),
            7 => self.registers.set_a(data),
            _ => panic!("Invalid register: {}", r_index),
        }
    }

    pub fn step(&mut self, bus: &mut MemoryBus) {
        let opcode = self.fetch_byte(bus);

        //XXYYYZZZ
        match opcode {
            0x00 => {}
            //LOAD
            0x40..=0x7F => {
                if opcode == 0x76 {
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
                self.alu(y, z, bus);
            }
            //LD r, d8
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x3E => {
                let d8 = self.imm8(bus);
                let r = (opcode >> 3) & 0x07; // works for these opcodes
                self.write_r(bus, r, d8);
            }
            //LD (HL), d8
            0x36 => {
                let d8 = self.imm8(bus);
                bus.write_byte(self.registers.get_hl(), d8);
            }
            //JP a16
            0xC3 => {
                let addr = self.imm16(bus);
                self.pc = addr;
            }
            //JR r8
            0x18 => {
                let off = self.imm8(bus) as i8;
                self.pc = ((self.pc as i32) + (off as i32)) as u16;
            }
            //JR cc, r8
            0x20 | 0x28 | 0x30 | 0x38 => {
                let off = self.imm8(bus) as i8;
                let take = match opcode {
                    0x20 => !self.registers.flag_z(),
                    0x28 => self.registers.flag_z(),
                    0x30 => !self.registers.flag_c(),
                    0x38 => self.registers.flag_c(),
                    _ => unreachable!(),
                };
                if take {
                    self.pc = ((self.pc as i32) + (off as i32)) as u16;
                }
            }
            //CALL a16
            0xCD => {
                let addr = self.imm16(bus);
                let ret = self.pc;
                self.push_u16(bus, ret);
                self.pc = addr;
            }
            //RET
            0xC9 => {
                let addr = self.pop_u16(bus);
                self.pc = addr;
            }
            //RET cc
            0xC0 | 0xC8 | 0xD0 | 0xD8 => {
                let take = match opcode {
                    0xC0 => !self.registers.flag_z(),
                    0xC8 => self.registers.flag_z(),
                    0xD0 => !self.registers.flag_c(),
                    0xD8 => self.registers.flag_c(),
                    _ => unreachable!(),
                };
                if take {
                    self.pc = self.pop_u16(bus);
                }
            }
            //PUSH
            0xC5 => self.push_u16(bus, self.registers.get_bc()),
            0xD5 => self.push_u16(bus, self.registers.get_de()),
            0xE5 => self.push_u16(bus, self.registers.get_hl()),
            0xF5 => self.push_u16(bus, self.registers.get_af()),
            //POP
            0xC1 => {
                let v = self.pop_u16(bus);
                self.registers.set_bc(v);
            }
            0xD1 => {
                let v = self.pop_u16(bus);
                self.registers.set_de(v);
            }
            0xE1 => {
                let v = self.pop_u16(bus);
                self.registers.set_hl(v);
            }
            0xF1 => {
                let v = self.pop_u16(bus);
                self.registers.set_af(v);
            } // set_af must mask low nibble of F
            _ => todo!("Opcode {:02X} not implemented", opcode),
        }
    }

    fn alu(&mut self, y: u8, z: u8, bus: &MemoryBus) {
        let value = self.read_r(bus, z);
        let carry_in = if self.registers.flag_c() { 1 } else { 0 };
        match y {
            //ADD A, r
            0 => self.add_a(value, 0),
            //ADC A, r
            1 => self.add_a(value, carry_in),
            //SUB A, r
            2 => self.sub_a(value, 0),
            //SBC A, r
            3 => self.sub_a(value, carry_in),
            //AND A, r
            4 => self.and_a(value),
            //XOR A, r
            5 => self.xor_a(value),
            //OR A, r
            6 => self.or_a(value),
            //CP A, r
            7 => self.cp_a(value),
            _ => unreachable!(),
        }
    }

    fn add_a(&mut self, value: u8, carry_in: u8) {
        let a = self.registers.get_a();
        let (tmp, c1) = a.overflowing_add(value);
        let (res, c2) = tmp.overflowing_add(carry_in);

        let half = ((a & 0x0F) + (value & 0x0F) + carry_in) > 0x0F;
        let carry = c1 || c2;

        self.registers.set_a(res);
        self.set_flags(res == 0, false, half, carry);
    }

    fn sub_a(&mut self, value: u8, carry_in: u8) {
        let a = self.registers.get_a();
        let (tmp, b1) = a.overflowing_sub(value);
        let (res, b2) = tmp.overflowing_sub(carry_in);

        let half = (a & 0x0F) < ((value & 0x0F) + carry_in);
        let carry = b1 || b2; // "borrow happened" => carry flag set

        self.registers.set_a(res);
        self.set_flags(res == 0, true, half, carry);
    }

    fn and_a(&mut self, value: u8) {
        let res = self.registers.get_a() & value;
        self.registers.set_a(res);
        self.set_flags(res == 0, false, true, false); // H=1 always for AND
    }

    fn xor_a(&mut self, value: u8) {
        let res = self.registers.get_a() ^ value;
        self.registers.set_a(res);
        self.set_flags(res == 0, false, false, false);
    }

    fn or_a(&mut self, value: u8) {
        let res = self.registers.get_a() | value;
        self.registers.set_a(res);
        self.set_flags(res == 0, false, false, false);
    }

    fn cp_a(&mut self, value: u8) {
        let a = self.registers.get_a();
        let res = a.wrapping_sub(value);
        let half = (a & 0x0F) < (value & 0x0F);
        let carry = a < value;

        // A bleibt unverändert!
        self.set_flags(res == 0, true, half, carry);
    }
}
