use crate::bus::MemoryBus;
use crate::register::Register;

pub struct Cpu {
    registers: Register,
    pc: u16,
    sp: u16,
    ime: bool, // Interrupt Master Enable
    ei_pending: bool,
    halted: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            registers: Register::new(),
            pc: 0x0100,
            sp: 0xFFFE,
            ime: false,
            ei_pending: false,
            halted: false,
        }
    }

    fn fetch_byte(&mut self, bus: &MemoryBus) -> u8 {
        let byte = bus.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    fn push_u8(&mut self, bus: &mut MemoryBus, value: u8) {
        self.sp = self.sp.wrapping_sub(1);
        bus.write_byte(self.sp, value);
    }

    fn pop_u8(&mut self, bus: &MemoryBus) -> u8 {
        let value = bus.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
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
        if self.ei_pending {
            self.ime = true;
            self.ei_pending = false;
        }
        if self.halted {
            return;
        }

        let opcode = self.fetch_byte(bus);

        //XXYYYZZZ
        match opcode {
            // ===== 8-bit load =====
            0x40..=0x7F => {
                if opcode == 0x76 {
                    self.halted = true;
                } else {
                    let dst = (opcode >> 3) & 0x07;
                    let src = opcode & 0x07;
                    let value = self.read_r(bus, src);
                    self.write_r(bus, dst, value);
                }
            }
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
                let dst = (opcode >> 3) & 0x07;
                let value = self.imm8(bus);
                self.write_r(bus, dst, value);
            }
            0x02 => bus.write_byte(self.registers.get_bc(), self.registers.get_a()),
            0x12 => bus.write_byte(self.registers.get_de(), self.registers.get_a()),
            0x22 => {
                let hl = self.registers.get_hl();
                bus.write_byte(hl, self.registers.get_a());
                self.registers.set_hl(hl.wrapping_add(1));
            }
            0x32 => {
                let hl = self.registers.get_hl();
                bus.write_byte(hl, self.registers.get_a());
                self.registers.set_hl(hl.wrapping_sub(1));
            }
            0x0A => self.registers.set_a(bus.read_byte(self.registers.get_bc())),
            0x1A => self.registers.set_a(bus.read_byte(self.registers.get_de())),
            0x2A => {
                let hl = self.registers.get_hl();
                self.registers.set_a(bus.read_byte(hl));
                self.registers.set_hl(hl.wrapping_add(1));
            }
            0x3A => {
                let hl = self.registers.get_hl();
                self.registers.set_a(bus.read_byte(hl));
                self.registers.set_hl(hl.wrapping_sub(1));
            }
            0xE0 => {
                let addr = 0xFF00u16 + self.imm8(bus) as u16;
                bus.write_byte(addr, self.registers.get_a());
            }
            0xF0 => {
                let addr = 0xFF00u16 + self.imm8(bus) as u16;
                self.registers.set_a(bus.read_byte(addr));
            }
            0xE2 => {
                let addr = 0xFF00u16 + self.registers.get_c() as u16;
                bus.write_byte(addr, self.registers.get_a());
            }
            0xF2 => {
                let addr = 0xFF00u16 + self.registers.get_c() as u16;
                self.registers.set_a(bus.read_byte(addr));
            }
            0xEA => {
                let addr = self.imm16(bus);
                bus.write_byte(addr, self.registers.get_a());
            }
            0xFA => {
                let addr = self.imm16(bus);
                self.registers.set_a(bus.read_byte(addr));
            }

            // ===== 16-bit load / stack =====
            0x01 => {
                let nn = self.imm16(bus);
                self.registers.set_bc(nn);
            }
            0x11 => {
                let nn = self.imm16(bus);
                self.registers.set_de(nn);
            }
            0x21 => {
                let nn = self.imm16(bus);
                self.registers.set_hl(nn);
            }
            0x31 => {
                self.sp = self.imm16(bus);
            }
            0x08 => {
                let addr = self.imm16(bus);
                bus.write_byte(addr, self.sp as u8);
                bus.write_byte(addr.wrapping_add(1), (self.sp >> 8) as u8);
            }
            0xF8 => {
                let e8 = self.imm8(bus) as i8;
                let sp = self.sp;
                let e_u = e8 as i16 as u16;
                let h = ((sp & 0x000F) + (e_u & 0x000F)) > 0x000F;
                let c = ((sp & 0x00FF) + (e_u & 0x00FF)) > 0x00FF;
                self.registers.set_hl((sp as i32 + e8 as i32) as u16);
                self.set_flags(false, false, h, c);
            }
            0xF9 => self.sp = self.registers.get_hl(),
            0xC5 => self.push_u16(bus, self.registers.get_bc()),
            0xD5 => self.push_u16(bus, self.registers.get_de()),
            0xE5 => self.push_u16(bus, self.registers.get_hl()),
            0xF5 => self.push_u16(bus, self.registers.get_af()),
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
            }

            // ===== 8-bit ALU =====
            0x80..=0xBF => {
                let y = (opcode >> 3) & 0x07;
                let z = opcode & 0x07;
                self.alu(y, z, bus);
            }
            0xC6 => {
                let v = self.imm8(bus);
                self.add_a(v, 0);
            }
            0xCE => {
                let c = if self.registers.flag_c() { 1 } else { 0 };
                let v = self.imm8(bus);
                self.add_a(v, c);
            }
            0xD6 => {
                let v = self.imm8(bus);
                self.sub_a(v, 0);
            }
            0xDE => {
                let c = if self.registers.flag_c() { 1 } else { 0 };
                let v = self.imm8(bus);
                self.sub_a(v, c);
            }
            0xE6 => {
                let v = self.imm8(bus);
                self.and_a(v);
            }
            0xEE => {
                let v = self.imm8(bus);
                self.xor_a(v);
            }
            0xF6 => {
                let v = self.imm8(bus);
                self.or_a(v);
            }
            0xFE => {
                let v = self.imm8(bus);
                self.cp_a(v);
            }

            // ===== INC/DEC =====
            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
                let r = (opcode >> 3) & 0x07;
                let v = self.read_r(bus, r);
                let res = v.wrapping_add(1);
                self.write_r(bus, r, res);
                self.registers.set_flag_z(res == 0);
                self.registers.set_flag_n(false);
                self.registers.set_flag_h((v & 0x0F) + 1 > 0x0F);
            }
            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
                let r = (opcode >> 3) & 0x07;
                let v = self.read_r(bus, r);
                let res = v.wrapping_sub(1);
                self.write_r(bus, r, res);
                self.registers.set_flag_z(res == 0);
                self.registers.set_flag_n(true);
                self.registers.set_flag_h((v & 0x0F) == 0);
            }

            // ===== 16-bit ALU =====
            0x03 => self.registers.set_bc(self.registers.get_bc().wrapping_add(1)),
            0x13 => self.registers.set_de(self.registers.get_de().wrapping_add(1)),
            0x23 => self.registers.set_hl(self.registers.get_hl().wrapping_add(1)),
            0x33 => self.sp = self.sp.wrapping_add(1),
            0x0B => self.registers.set_bc(self.registers.get_bc().wrapping_sub(1)),
            0x1B => self.registers.set_de(self.registers.get_de().wrapping_sub(1)),
            0x2B => self.registers.set_hl(self.registers.get_hl().wrapping_sub(1)),
            0x3B => self.sp = self.sp.wrapping_sub(1),
            0x09 => self.add_hl(self.registers.get_bc()),
            0x19 => self.add_hl(self.registers.get_de()),
            0x29 => self.add_hl(self.registers.get_hl()),
            0x39 => self.add_hl(self.sp),
            0xE8 => {
                let e8 = self.imm8(bus) as i8;
                let sp = self.sp;
                let e_u = e8 as i16 as u16;
                let h = ((sp & 0x000F) + (e_u & 0x000F)) > 0x000F;
                let c = ((sp & 0x00FF) + (e_u & 0x00FF)) > 0x00FF;
                self.sp = (sp as i32 + e8 as i32) as u16;
                self.set_flags(false, false, h, c);
            }

            // ===== rotates / misc =====
            0x07 => {
                let a = self.registers.get_a();
                let c = (a & 0x80) != 0;
                self.registers.set_a((a << 1) | if c { 1 } else { 0 });
                self.set_flags(false, false, false, c);
            }
            0x17 => {
                let a = self.registers.get_a();
                let cin = if self.registers.flag_c() { 1 } else { 0 };
                let c = (a & 0x80) != 0;
                self.registers.set_a((a << 1) | cin);
                self.set_flags(false, false, false, c);
            }
            0x0F => {
                let a = self.registers.get_a();
                let c = (a & 0x01) != 0;
                self.registers.set_a((a >> 1) | if c { 0x80 } else { 0 });
                self.set_flags(false, false, false, c);
            }
            0x1F => {
                let a = self.registers.get_a();
                let cin = if self.registers.flag_c() { 0x80 } else { 0 };
                let c = (a & 0x01) != 0;
                self.registers.set_a((a >> 1) | cin);
                self.set_flags(false, false, false, c);
            }
            0x00 => {}
            0x10 => {
                let _ = self.imm8(bus);
                self.halted = true;
            }
            0x27 => self.daa(),
            0x2F => {
                self.registers.set_a(!self.registers.get_a());
                self.registers.set_flag_n(true);
                self.registers.set_flag_h(true);
            }
            0x37 => {
                self.registers.set_flag_n(false);
                self.registers.set_flag_h(false);
                self.registers.set_flag_c(true);
            }
            0x3F => {
                let c = !self.registers.flag_c();
                self.registers.set_flag_n(false);
                self.registers.set_flag_h(false);
                self.registers.set_flag_c(c);
            }
            0xF3 => self.ime = false,
            0xFB => self.ei_pending = true,

            // ===== jumps / calls / returns =====
            0x18 => {
                let e = self.imm8(bus) as i8;
                self.jr(e);
            }
            0x20 => {
                let e = self.imm8(bus) as i8;
                if !self.registers.flag_z() {
                    self.jr(e);
                }
            }
            0x28 => {
                let e = self.imm8(bus) as i8;
                if self.registers.flag_z() {
                    self.jr(e);
                }
            }
            0x30 => {
                let e = self.imm8(bus) as i8;
                if !self.registers.flag_c() {
                    self.jr(e);
                }
            }
            0x38 => {
                let e = self.imm8(bus) as i8;
                if self.registers.flag_c() {
                    self.jr(e);
                }
            }
            0xC3 => self.pc = self.imm16(bus),
            0xC2 => {
                let addr = self.imm16(bus);
                if !self.registers.flag_z() {
                    self.pc = addr;
                }
            }
            0xCA => {
                let addr = self.imm16(bus);
                if self.registers.flag_z() {
                    self.pc = addr;
                }
            }
            0xD2 => {
                let addr = self.imm16(bus);
                if !self.registers.flag_c() {
                    self.pc = addr;
                }
            }
            0xDA => {
                let addr = self.imm16(bus);
                if self.registers.flag_c() {
                    self.pc = addr;
                }
            }
            0xE9 => self.pc = self.registers.get_hl(),
            0xCD => {
                let addr = self.imm16(bus);
                let ret = self.pc;
                self.push_u16(bus, ret);
                self.pc = addr;
            }
            0xC4 => {
                let addr = self.imm16(bus);
                if !self.registers.flag_z() {
                    let ret = self.pc;
                    self.push_u16(bus, ret);
                    self.pc = addr;
                }
            }
            0xCC => {
                let addr = self.imm16(bus);
                if self.registers.flag_z() {
                    let ret = self.pc;
                    self.push_u16(bus, ret);
                    self.pc = addr;
                }
            }
            0xD4 => {
                let addr = self.imm16(bus);
                if !self.registers.flag_c() {
                    let ret = self.pc;
                    self.push_u16(bus, ret);
                    self.pc = addr;
                }
            }
            0xDC => {
                let addr = self.imm16(bus);
                if self.registers.flag_c() {
                    let ret = self.pc;
                    self.push_u16(bus, ret);
                    self.pc = addr;
                }
            }
            0xC9 => self.pc = self.pop_u16(bus),
            0xD9 => {
                self.pc = self.pop_u16(bus);
                self.ime = true;
            }
            0xC0 => {
                if !self.registers.flag_z() {
                    self.pc = self.pop_u16(bus);
                }
            }
            0xC8 => {
                if self.registers.flag_z() {
                    self.pc = self.pop_u16(bus);
                }
            }
            0xD0 => {
                if !self.registers.flag_c() {
                    self.pc = self.pop_u16(bus);
                }
            }
            0xD8 => {
                if self.registers.flag_c() {
                    self.pc = self.pop_u16(bus);
                }
            }
            0xC7 => self.rst(bus, 0x00),
            0xCF => self.rst(bus, 0x08),
            0xD7 => self.rst(bus, 0x10),
            0xDF => self.rst(bus, 0x18),
            0xE7 => self.rst(bus, 0x20),
            0xEF => self.rst(bus, 0x28),
            0xF7 => self.rst(bus, 0x30),
            0xFF => self.rst(bus, 0x38),

            // ===== CB-prefix =====
            0xCB => {
                let cb = self.fetch_byte(bus);
                self.exec_cb(bus, cb);
            }

            _ => println!("Opcode {:02X} not implemented (illegal or reserved)", opcode),
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

    fn add_hl(&mut self, value: u16) {
        let hl = self.registers.get_hl();
        let res = hl.wrapping_add(value);
        let half = ((hl & 0x0FFF) + (value & 0x0FFF)) > 0x0FFF;
        let carry = (hl as u32 + value as u32) > 0xFFFF;
        self.registers.set_hl(res);
        self.registers.set_flag_n(false);
        self.registers.set_flag_h(half);
        self.registers.set_flag_c(carry);
    }

    fn jr(&mut self, offset: i8) {
        self.pc = (self.pc as i32 + offset as i32) as u16;
    }

    fn rst(&mut self, bus: &mut MemoryBus, addr: u16) {
        let ret = self.pc;
        self.push_u16(bus, ret);
        self.pc = addr;
    }

    fn daa(&mut self) {
        let mut a = self.registers.get_a();
        let n = self.registers.flag_n();
        let mut c = self.registers.flag_c();
        let h = self.registers.flag_h();

        let mut adjust = 0u8;
        if !n {
            if c || a > 0x99 {
                adjust |= 0x60;
                c = true;
            }
            if h || (a & 0x0F) > 0x09 {
                adjust |= 0x06;
            }
            a = a.wrapping_add(adjust);
        } else {
            if c {
                adjust |= 0x60;
            }
            if h {
                adjust |= 0x06;
            }
            a = a.wrapping_sub(adjust);
        }

        self.registers.set_a(a);
        self.registers.set_flag_z(a == 0);
        self.registers.set_flag_h(false);
        self.registers.set_flag_c(c);
    }

    fn exec_cb(&mut self, bus: &mut MemoryBus, opcode: u8) {
        let x = opcode >> 6;
        let y = (opcode >> 3) & 0x07;
        let z = opcode & 0x07;

        match x {
            0 => {
                let value = self.read_r(bus, z);
                let (res, carry) = match y {
                    0 => {
                        let c = (value & 0x80) != 0;
                        ((value << 1) | if c { 1 } else { 0 }, c)
                    }
                    1 => {
                        let c = (value & 0x01) != 0;
                        ((value >> 1) | if c { 0x80 } else { 0 }, c)
                    }
                    2 => {
                        let c = (value & 0x80) != 0;
                        let cin = if self.registers.flag_c() { 1 } else { 0 };
                        ((value << 1) | cin, c)
                    }
                    3 => {
                        let c = (value & 0x01) != 0;
                        let cin = if self.registers.flag_c() { 0x80 } else { 0 };
                        ((value >> 1) | cin, c)
                    }
                    4 => (value << 1, (value & 0x80) != 0),
                    5 => ((value >> 1) | (value & 0x80), (value & 0x01) != 0),
                    6 => ((value >> 4) | (value << 4), false),
                    7 => (value >> 1, (value & 0x01) != 0),
                    _ => unreachable!(),
                };

                self.write_r(bus, z, res);
                self.set_flags(res == 0, false, false, carry);
            }
            1 => {
                let value = self.read_r(bus, z);
                let bit_set = (value & (1 << y)) != 0;
                self.registers.set_flag_z(!bit_set);
                self.registers.set_flag_n(false);
                self.registers.set_flag_h(true);
            }
            2 => {
                let value = self.read_r(bus, z);
                self.write_r(bus, z, value & !(1 << y));
            }
            3 => {
                let value = self.read_r(bus, z);
                self.write_r(bus, z, value | (1 << y));
            }
            _ => unreachable!(),
        }
    }
}
