use crate::bus::MemoryBus;
use crate::interrupts::InterruptType;
use crate::register::Register;

pub struct Cpu {
    registers: Register,
    pc: u16,
    sp: u16,
    ime: bool,        //Interrupt Master Enable
    ei_pending: bool, //EI delayed by one instruction, we need to track if pending
    halted: bool,
    step_cycles: u32,

    halt_bug: bool,
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
            step_cycles: 0,

            halt_bug: false,
        }
    }

    fn m_cycle(&mut self, bus: &mut MemoryBus) {
        bus.tick(4);
        self.step_cycles += 4;
    }

    fn read_byte_timed(&mut self, bus: &mut MemoryBus, addr: u16) -> u8 {
        let byte = bus.read_byte(addr);
        self.m_cycle(bus);
        byte
    }

    fn write_byte_timed(&mut self, bus: &mut MemoryBus, addr: u16, value: u8) {
        bus.write_byte(addr, value);
        self.m_cycle(bus);
    }

    fn fetch_byte(&mut self, bus: &mut MemoryBus) -> u8 {
        let byte = self.read_byte_timed(bus, self.pc);
        if self.halt_bug {
            self.halt_bug = false;
        } else {
            self.pc = self.pc.wrapping_add(1);
        }
        byte
    }

    fn push_u8(&mut self, bus: &mut MemoryBus, value: u8) {
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte_timed(bus, self.sp, value);
    }

    fn pop_u8(&mut self, bus: &mut MemoryBus) -> u8 {
        let value = self.read_byte_timed(bus, self.sp);
        self.sp = self.sp.wrapping_add(1);
        value
    }

    fn push_u16(&mut self, bus: &mut MemoryBus, value: u16) {
        let hi = (value >> 8) as u8;
        let lo = value as u8;
        self.push_u8(bus, hi);
        self.push_u8(bus, lo);
    }
    fn pop_u16(&mut self, bus: &mut MemoryBus) -> u16 {
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

    fn imm8(&mut self, bus: &mut MemoryBus) -> u8 {
        self.fetch_byte(bus)
    }

    fn imm16(&mut self, bus: &mut MemoryBus) -> u16 {
        let lo = self.fetch_byte(bus) as u16;
        let hi = self.fetch_byte(bus) as u16;
        (hi << 8) | lo
    }

    pub fn read_r(&mut self, bus: &mut MemoryBus, r_index: u8) -> u8 {
        match r_index {
            0 => self.registers.get_b(),
            1 => self.registers.get_c(),
            2 => self.registers.get_d(),
            3 => self.registers.get_e(),
            4 => self.registers.get_h(),
            5 => self.registers.get_l(),
            6 => self.read_byte_timed(bus, self.registers.get_hl()),
            7 => self.registers.get_a(),
            _ => {
                eprintln!("read_r: invalid register index {} - returning 0", r_index);
                0
            }
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
            6 => self.write_byte_timed(bus, self.registers.get_hl(), data),
            7 => self.registers.set_a(data),
            _ => eprintln!("write_r: invalid register index {} - write ignored", r_index),
        }
    }

    //TODO: fix interrupts, halting, and add timers
    pub fn step(&mut self, bus: &mut MemoryBus) -> u32 {
        self.step_cycles = 0;

        let enable_ime_after_step = self.ei_pending;
        self.ei_pending = false;

        let pending = bus.pending_interrupts();

        if self.halted {
            if pending != 0 {
                self.halted = false;
            } else {
                self.m_cycle(bus);
                if enable_ime_after_step {
                    self.ime = true;
                }
                return self.step_cycles;
            }
        }

        if self.ime && pending != 0 {
            self.halt_bug = false;
            let irq = match InterruptType::highest_priority_from_pending(pending) {
                Some(i) => i,
                None => {
                    eprintln!("step(): pending interrupts non-zero but no known IRQ bit found: {:#04x}", pending);
                    if enable_ime_after_step { self.ime = true; }
                    return self.step_cycles;
                }
            };

            self.ime = false;
            bus.acknowledge_interrupt(irq.bit());

            self.m_cycle(bus);
            self.m_cycle(bus);
            self.m_cycle(bus);
            self.push_u16(bus, self.pc);
            self.pc = irq.vector();
            if enable_ime_after_step {
                self.ime = true;
            }
            return self.step_cycles;
        }

        let opcode = self.fetch_byte(bus);
        let mut branch_taken = false;
        let mut cb_opcode: Option<u8> = None;

        //XXYYYZZZ
        match opcode {
            // ===== 8-bit load =====
            0x40..=0x7F => {
                if opcode == 0x76 {
                    if (self.ime == false) && bus.pending_interrupts() != 0 {
                        self.halt_bug = true;
                    } else {
                        self.halted = true;
                    }
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
            0x02 => self.write_byte_timed(bus, self.registers.get_bc(), self.registers.get_a()),
            0x12 => self.write_byte_timed(bus, self.registers.get_de(), self.registers.get_a()),
            0x22 => {
                let hl = self.registers.get_hl();
                self.write_byte_timed(bus, hl, self.registers.get_a());
                self.registers.set_hl(hl.wrapping_add(1));
            }
            0x32 => {
                let hl = self.registers.get_hl();
                self.write_byte_timed(bus, hl, self.registers.get_a());
                self.registers.set_hl(hl.wrapping_sub(1));
            }
            0x0A => {
                let v = self.read_byte_timed(bus, self.registers.get_bc());
                self.registers.set_a(v);
            }
            0x1A => {
                let v = self.read_byte_timed(bus, self.registers.get_de());
                self.registers.set_a(v);
            }
            0x2A => {
                let hl = self.registers.get_hl();
                let v = self.read_byte_timed(bus, hl);
                self.registers.set_a(v);
                self.registers.set_hl(hl.wrapping_add(1));
            }
            0x3A => {
                let hl = self.registers.get_hl();
                let v = self.read_byte_timed(bus, hl);
                self.registers.set_a(v);
                self.registers.set_hl(hl.wrapping_sub(1));
            }
            0xE0 => {
                let addr = 0xFF00u16 + self.imm8(bus) as u16;
                self.write_byte_timed(bus, addr, self.registers.get_a());
            }
            0xF0 => {
                let addr = 0xFF00u16 + self.imm8(bus) as u16;
                let v = self.read_byte_timed(bus, addr);
                self.registers.set_a(v);
            }
            0xE2 => {
                let addr = 0xFF00u16 + self.registers.get_c() as u16;
                self.write_byte_timed(bus, addr, self.registers.get_a());
            }
            0xF2 => {
                let addr = 0xFF00u16 + self.registers.get_c() as u16;
                let v = self.read_byte_timed(bus, addr);
                self.registers.set_a(v);
            }
            0xEA => {
                let addr = self.imm16(bus);
                self.write_byte_timed(bus, addr, self.registers.get_a());
            }
            0xFA => {
                let addr = self.imm16(bus);
                let v = self.read_byte_timed(bus, addr);
                self.registers.set_a(v);
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
                self.write_byte_timed(bus, addr, self.sp as u8);
                self.write_byte_timed(bus, addr.wrapping_add(1), (self.sp >> 8) as u8);
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
            0x03 => self
                .registers
                .set_bc(self.registers.get_bc().wrapping_add(1)),
            0x13 => self
                .registers
                .set_de(self.registers.get_de().wrapping_add(1)),
            0x23 => self
                .registers
                .set_hl(self.registers.get_hl().wrapping_add(1)),
            0x33 => self.sp = self.sp.wrapping_add(1),
            0x0B => self
                .registers
                .set_bc(self.registers.get_bc().wrapping_sub(1)),
            0x1B => self
                .registers
                .set_de(self.registers.get_de().wrapping_sub(1)),
            0x2B => self
                .registers
                .set_hl(self.registers.get_hl().wrapping_sub(1)),
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
                // STOP (DMG): placeholder behavior until full joypad/speed-switch support.
                // Do not enter permanent halt state.
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
                    branch_taken = true;
                }
            }
            0x28 => {
                let e = self.imm8(bus) as i8;
                if self.registers.flag_z() {
                    self.jr(e);
                    branch_taken = true;
                }
            }
            0x30 => {
                let e = self.imm8(bus) as i8;
                if !self.registers.flag_c() {
                    self.jr(e);
                    branch_taken = true;
                }
            }
            0x38 => {
                let e = self.imm8(bus) as i8;
                if self.registers.flag_c() {
                    self.jr(e);
                    branch_taken = true;
                }
            }
            0xC3 => self.pc = self.imm16(bus),
            0xC2 => {
                let addr = self.imm16(bus);
                if !self.registers.flag_z() {
                    self.pc = addr;
                    branch_taken = true;
                }
            }
            0xCA => {
                let addr = self.imm16(bus);
                if self.registers.flag_z() {
                    self.pc = addr;
                    branch_taken = true;
                }
            }
            0xD2 => {
                let addr = self.imm16(bus);
                if !self.registers.flag_c() {
                    self.pc = addr;
                    branch_taken = true;
                }
            }
            0xDA => {
                let addr = self.imm16(bus);
                if self.registers.flag_c() {
                    self.pc = addr;
                    branch_taken = true;
                }
            }
            0xE9 => self.pc = self.registers.get_hl(),
            0xCD => {
                let addr = self.imm16(bus);
                let ret = self.pc;
                self.push_u16(bus, ret);
                self.pc = addr;
                branch_taken = true;
            }
            0xC4 => {
                let addr = self.imm16(bus);
                if !self.registers.flag_z() {
                    let ret = self.pc;
                    self.push_u16(bus, ret);
                    self.pc = addr;
                    branch_taken = true;
                }
            }
            0xCC => {
                let addr = self.imm16(bus);
                if self.registers.flag_z() {
                    let ret = self.pc;
                    self.push_u16(bus, ret);
                    self.pc = addr;
                    branch_taken = true;
                }
            }
            0xD4 => {
                let addr = self.imm16(bus);
                if !self.registers.flag_c() {
                    let ret = self.pc;
                    self.push_u16(bus, ret);
                    self.pc = addr;
                    branch_taken = true;
                }
            }
            0xDC => {
                let addr = self.imm16(bus);
                if self.registers.flag_c() {
                    let ret = self.pc;
                    self.push_u16(bus, ret);
                    self.pc = addr;
                    branch_taken = true;
                }
            }
            0xC9 => {
                self.pc = self.pop_u16(bus);
                branch_taken = true;
            }
            0xD9 => {
                self.pc = self.pop_u16(bus);
                self.ime = true;
                branch_taken = true;
            }
            0xC0 => {
                if !self.registers.flag_z() {
                    self.pc = self.pop_u16(bus);
                    branch_taken = true;
                }
            }
            0xC8 => {
                if self.registers.flag_z() {
                    self.pc = self.pop_u16(bus);
                    branch_taken = true;
                }
            }
            0xD0 => {
                if !self.registers.flag_c() {
                    self.pc = self.pop_u16(bus);
                    branch_taken = true;
                }
            }
            0xD8 => {
                if self.registers.flag_c() {
                    self.pc = self.pop_u16(bus);
                    branch_taken = true;
                }
            }
            0xC7 => {
                self.rst(bus, 0x00);
                branch_taken = true;
            }
            0xCF => {
                self.rst(bus, 0x08);
                branch_taken = true;
            }
            0xD7 => {
                self.rst(bus, 0x10);
                branch_taken = true;
            }
            0xDF => {
                self.rst(bus, 0x18);
                branch_taken = true;
            }
            0xE7 => {
                self.rst(bus, 0x20);
                branch_taken = true;
            }
            0xEF => {
                self.rst(bus, 0x28);
                branch_taken = true;
            }
            0xF7 => {
                self.rst(bus, 0x30);
                branch_taken = true;
            }
            0xFF => {
                self.rst(bus, 0x38);
                branch_taken = true;
            }

            // ===== CB-prefix =====
            0xCB => {
                let cb = self.fetch_byte(bus);
                self.exec_cb(bus, cb);
                cb_opcode = Some(cb);
            }

            _ => println!(
                "Opcode {:02X} not implemented (illegal or reserved)",
                opcode
            ),
        }

        let target_cycles = Self::target_cycles(opcode, branch_taken, cb_opcode);
        while self.step_cycles < target_cycles {
            self.m_cycle(bus);
        }

        if enable_ime_after_step {
            self.ime = true;
        }

        self.step_cycles
    }

    fn target_cycles(opcode: u8, branch_taken: bool, cb_opcode: Option<u8>) -> u32 {
        match opcode {
            0x00 | 0x07 | 0x0F | 0x17 | 0x1F | 0x27 | 0x2F | 0x37 | 0x3F | 0x76 | 0xF3 | 0xFB
            | 0xE9 => 4,

            0x40..=0x7F => {
                let dst = (opcode >> 3) & 0x07;
                let src = opcode & 0x07;
                if dst == 6 || src == 6 { 8 } else { 4 }
            }

            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x3E => 8,
            0x36 => 12,
            0x02 | 0x0A | 0x12 | 0x1A | 0x22 | 0x2A | 0x32 | 0x3A | 0xE2 | 0xF2 => 8,
            0xE0 | 0xF0 => 12,
            0xEA | 0xFA => 16,

            0x01 | 0x11 | 0x21 | 0x31 => 12,
            0x08 => 20,
            0xF8 => 12,
            0xF9 => 8,
            0xC5 | 0xD5 | 0xE5 | 0xF5 => 16,
            0xC1 | 0xD1 | 0xE1 | 0xF1 => 12,

            0x80..=0xBF => {
                let src = opcode & 0x07;
                if src == 6 { 8 } else { 4 }
            }
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => 8,

            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x3C => 4,
            0x34 => 12,
            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x3D => 4,
            0x35 => 12,

            0x03 | 0x13 | 0x23 | 0x33 | 0x0B | 0x1B | 0x2B | 0x3B | 0x09 | 0x19 | 0x29 | 0x39 => 8,
            0xE8 => 16,

            0x10 => 4,

            0x18 => 12,
            0x20 | 0x28 | 0x30 | 0x38 => {
                if branch_taken {
                    12
                } else {
                    8
                }
            }
            0xC3 => 16,
            0xC2 | 0xCA | 0xD2 | 0xDA => {
                if branch_taken {
                    16
                } else {
                    12
                }
            }
            0xCD => 24,
            0xC4 | 0xCC | 0xD4 | 0xDC => {
                if branch_taken {
                    24
                } else {
                    12
                }
            }
            0xC9 | 0xD9 => 16,
            0xC0 | 0xC8 | 0xD0 | 0xD8 => {
                if branch_taken {
                    20
                } else {
                    8
                }
            }
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => 16,

            0xCB => {
                let cb = cb_opcode.unwrap_or(0);
                let x = cb >> 6;
                let z = cb & 0x07;
                if z == 6 {
                    if x == 1 { 12 } else { 16 }
                } else {
                    8
                }
            }

            _ => 4,
        }
    }

    fn alu(&mut self, y: u8, z: u8, bus: &mut MemoryBus) {
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
