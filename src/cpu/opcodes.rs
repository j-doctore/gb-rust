use crate::membus::MemoryBus;
use super::alu as alu_ops;

// XXYYYZZZ
pub fn execute(cpu: &mut super::Cpu, bus: &mut MemoryBus, opcode: u8) {
	match opcode {
		// ===== 8-bit load =====
		0x40..=0x7F => {
			if opcode == 0x76 {
				cpu.halted = true;
			} else {
				let dst = (opcode >> 3) & 0x07;
				let src = opcode & 0x07;
				let value = cpu.read_reg(bus, src);
				cpu.write_reg(bus, dst, value);
			}
		}
		0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
			let dst = (opcode >> 3) & 0x07;
			let value = cpu.imm8(bus);
			cpu.write_reg(bus, dst, value);
		}
		0x02 => cpu.write_byte(bus, cpu.registers.get_bc(), cpu.registers.get_a()),
		0x12 => cpu.write_byte(bus, cpu.registers.get_de(), cpu.registers.get_a()),
		0x22 => {
			let hl = cpu.registers.get_hl();
			cpu.write_byte(bus, hl, cpu.registers.get_a());
			cpu.registers.set_hl(hl.wrapping_add(1));
		}
		0x32 => {
			let hl = cpu.registers.get_hl();
			cpu.write_byte(bus, hl, cpu.registers.get_a());
			cpu.registers.set_hl(hl.wrapping_sub(1));
		}
		0x0A => {
			let v = cpu.read_byte(bus, cpu.registers.get_bc());
			cpu.registers.set_a(v);
		}
		0x1A => {
			let v = cpu.read_byte(bus, cpu.registers.get_de());
			cpu.registers.set_a(v);
		}
		0x2A => {
			let hl = cpu.registers.get_hl();
			let v = cpu.read_byte(bus, hl);
			cpu.registers.set_a(v);
			cpu.registers.set_hl(hl.wrapping_add(1));
		}
		0x3A => {
			let hl = cpu.registers.get_hl();
			let v = cpu.read_byte(bus, hl);
			cpu.registers.set_a(v);
			cpu.registers.set_hl(hl.wrapping_sub(1));
		}
		0xE0 => {
			let addr = 0xFF00u16 + cpu.imm8(bus) as u16;
			cpu.write_byte(bus, addr, cpu.registers.get_a());
		}
		0xF0 => {
			let addr = 0xFF00u16 + cpu.imm8(bus) as u16;
			let v = cpu.read_byte(bus, addr);
			cpu.registers.set_a(v);
		}
		0xE2 => {
			let addr = 0xFF00u16 + cpu.registers.get_c() as u16;
			cpu.write_byte(bus, addr, cpu.registers.get_a());
		}
		0xF2 => {
			let addr = 0xFF00u16 + cpu.registers.get_c() as u16;
			let v = cpu.read_byte(bus, addr);
			cpu.registers.set_a(v);
		}
		0xEA => {
			let addr = cpu.imm16(bus);
			cpu.write_byte(bus, addr, cpu.registers.get_a());
		}
		0xFA => {
			let addr = cpu.imm16(bus);
			let v = cpu.read_byte(bus, addr);
			cpu.registers.set_a(v);
		}

		// ===== 16-bit load / stack =====
		0x01 => {
			let nn = cpu.imm16(bus);
			cpu.registers.set_bc(nn);
		}
		0x11 => {
			let nn = cpu.imm16(bus);
			cpu.registers.set_de(nn);
		}
		0x21 => {
			let nn = cpu.imm16(bus);
			cpu.registers.set_hl(nn);
		}
		0x31 => {
			cpu.sp = cpu.imm16(bus);
		}
		0x08 => {
			let addr = cpu.imm16(bus);
			cpu.write_byte(bus, addr, cpu.sp as u8);
			cpu.write_byte(bus, addr.wrapping_add(1), (cpu.sp >> 8) as u8);
		}
		0xF8 => {
			let e8 = cpu.imm8(bus) as i8;
			let sp = cpu.sp;
			let e_u = e8 as i16 as u16;
			let h = ((sp & 0x000F) + (e_u & 0x000F)) > 0x000F;
			let c = ((sp & 0x00FF) + (e_u & 0x00FF)) > 0x00FF;
			cpu.registers.set_hl((sp as i32 + e8 as i32) as u16);
			cpu.set_flags(false, false, h, c);
		}
		0xF9 => cpu.sp = cpu.registers.get_hl(),
		0xC5 => cpu.push_u16(bus, cpu.registers.get_bc()),
		0xD5 => cpu.push_u16(bus, cpu.registers.get_de()),
		0xE5 => cpu.push_u16(bus, cpu.registers.get_hl()),
		0xF5 => cpu.push_u16(bus, cpu.registers.get_af()),
		0xC1 => {
			let v = cpu.pop_u16(bus);
			cpu.registers.set_bc(v);
		}
		0xD1 => {
			let v = cpu.pop_u16(bus);
			cpu.registers.set_de(v);
		}
		0xE1 => {
			let v = cpu.pop_u16(bus);
			cpu.registers.set_hl(v);
		}
		0xF1 => {
			let v = cpu.pop_u16(bus);
			cpu.registers.set_af(v);
		}

		// ===== 8-bit ALU =====
		0x80..=0xBF => {
			let y = (opcode >> 3) & 0x07;
			let z = opcode & 0x07;
			alu_ops::alu(cpu, bus, y, z);
		}
		0xC6 => {
			let v = cpu.imm8(bus);
			alu_ops::add_a(&mut cpu.registers, v, 0);
		}
		0xCE => {
			let c = if cpu.registers.flag_c() { 1 } else { 0 };
			let v = cpu.imm8(bus);
			alu_ops::add_a(&mut cpu.registers, v, c);
		}
		0xD6 => {
			let v = cpu.imm8(bus);
			alu_ops::sub_a(&mut cpu.registers, v, 0);
		}
		0xDE => {
			let c = if cpu.registers.flag_c() { 1 } else { 0 };
			let v = cpu.imm8(bus);
			alu_ops::sub_a(&mut cpu.registers, v, c);
		}
		0xE6 => {
			let v = cpu.imm8(bus);
			alu_ops::and_a(&mut cpu.registers, v);
		}
		0xEE => {
			let v = cpu.imm8(bus);
			alu_ops::xor_a(&mut cpu.registers, v);
		}
		0xF6 => {
			let v = cpu.imm8(bus);
			alu_ops::or_a(&mut cpu.registers, v);
		}
		0xFE => {
			let v = cpu.imm8(bus);
			alu_ops::cp_a(&mut cpu.registers, v);
		}

		// ===== INC/DEC =====
		0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
			let r = (opcode >> 3) & 0x07;
			let v = cpu.read_reg(bus, r);
			let res = v.wrapping_add(1);
			cpu.write_reg(bus, r, res);
			cpu.registers.set_flag_z(res == 0);
			cpu.registers.set_flag_n(false);
			cpu.registers.set_flag_h((v & 0x0F) + 1 > 0x0F);
		}
		0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
			let r = (opcode >> 3) & 0x07;
			let v = cpu.read_reg(bus, r);
			let res = v.wrapping_sub(1);
			cpu.write_reg(bus, r, res);
			cpu.registers.set_flag_z(res == 0);
			cpu.registers.set_flag_n(true);
			cpu.registers.set_flag_h((v & 0x0F) == 0);
		}

		// ===== 16-bit ALU =====
		0x03 => cpu.registers.set_bc(cpu.registers.get_bc().wrapping_add(1)),
		0x13 => cpu.registers.set_de(cpu.registers.get_de().wrapping_add(1)),
		0x23 => cpu.registers.set_hl(cpu.registers.get_hl().wrapping_add(1)),
		0x33 => cpu.sp = cpu.sp.wrapping_add(1),
		0x0B => cpu.registers.set_bc(cpu.registers.get_bc().wrapping_sub(1)),
		0x1B => cpu.registers.set_de(cpu.registers.get_de().wrapping_sub(1)),
		0x2B => cpu.registers.set_hl(cpu.registers.get_hl().wrapping_sub(1)),
		0x3B => cpu.sp = cpu.sp.wrapping_sub(1),
		0x09 => {
			let v = cpu.registers.get_bc();
			alu_ops::add_hl(&mut cpu.registers, v);
		}
		0x19 => {
			let v = cpu.registers.get_de();
			alu_ops::add_hl(&mut cpu.registers, v);
		}
		0x29 => {
			let v = cpu.registers.get_hl();
			alu_ops::add_hl(&mut cpu.registers, v);
		}
		0x39 => alu_ops::add_hl(&mut cpu.registers, cpu.sp),
		0xE8 => {
			let e8 = cpu.imm8(bus) as i8;
			let sp = cpu.sp;
			let e_u = e8 as i16 as u16;
			let h = ((sp & 0x000F) + (e_u & 0x000F)) > 0x000F;
			let c = ((sp & 0x00FF) + (e_u & 0x00FF)) > 0x00FF;
			cpu.sp = (sp as i32 + e8 as i32) as u16;
			cpu.set_flags(false, false, h, c);
		}

		// ===== rotates / misc =====
		0x07 => {
			let a = cpu.registers.get_a();
			let c = (a & 0x80) != 0;
			cpu.registers.set_a((a << 1) | if c { 1 } else { 0 });
			cpu.set_flags(false, false, false, c);
		}
		0x17 => {
			let a = cpu.registers.get_a();
			let cin = if cpu.registers.flag_c() { 1 } else { 0 };
			let c = (a & 0x80) != 0;
			cpu.registers.set_a((a << 1) | cin);
			cpu.set_flags(false, false, false, c);
		}
		0x0F => {
			let a = cpu.registers.get_a();
			let c = (a & 0x01) != 0;
			cpu.registers.set_a((a >> 1) | if c { 0x80 } else { 0 });
			cpu.set_flags(false, false, false, c);
		}
		0x1F => {
			let a = cpu.registers.get_a();
			let cin = if cpu.registers.flag_c() { 0x80 } else { 0 };
			let c = (a & 0x01) != 0;
			cpu.registers.set_a((a >> 1) | cin);
			cpu.set_flags(false, false, false, c);
		}
		0x00 => {}
		0x10 => {
			let _ = cpu.imm8(bus);
		}
		0x27 => alu_ops::daa(&mut cpu.registers),
		0x2F => {
			cpu.registers.set_a(!cpu.registers.get_a());
			cpu.registers.set_flag_n(true);
			cpu.registers.set_flag_h(true);
		}
		0x37 => {
			cpu.registers.set_flag_n(false);
			cpu.registers.set_flag_h(false);
			cpu.registers.set_flag_c(true);
		}
		0x3F => {
			let c = !cpu.registers.flag_c();
			cpu.registers.set_flag_n(false);
			cpu.registers.set_flag_h(false);
			cpu.registers.set_flag_c(c);
		}
		0xF3 => cpu.ime = false,
		0xFB => cpu.ei_delay = 2,

		// ===== jumps / calls / returns =====
		0x18 => {
			let e = cpu.imm8(bus) as i8;
			jr(cpu, e);
		}
		0x20 => {
			let e = cpu.imm8(bus) as i8;
			if !cpu.registers.flag_z() {
                jr(cpu, e);
			}
		}
		0x28 => {
			let e = cpu.imm8(bus) as i8;
			if cpu.registers.flag_z() {
				jr(cpu, e);
			}
		}
		0x30 => {
			let e = cpu.imm8(bus) as i8;
			if !cpu.registers.flag_c() {
				jr(cpu, e);
			}
		}
		0x38 => {
			let e = cpu.imm8(bus) as i8;
			if cpu.registers.flag_c() {
				jr(cpu, e);
			}
		}
		0xC3 => cpu.pc = cpu.imm16(bus),
		0xC2 => {
			let addr = cpu.imm16(bus);
			if !cpu.registers.flag_z() {
				cpu.pc = addr;
			}
		}
		0xCA => {
			let addr = cpu.imm16(bus);
			if cpu.registers.flag_z() {
				cpu.pc = addr;
			}
		}
		0xD2 => {
			let addr = cpu.imm16(bus);
			if !cpu.registers.flag_c() {
				cpu.pc = addr;
			}
		}
		0xDA => {
			let addr = cpu.imm16(bus);
			if cpu.registers.flag_c() {
				cpu.pc = addr;
			}
		}
		0xE9 => cpu.pc = cpu.registers.get_hl(),
		0xCD => {
			let addr = cpu.imm16(bus);
			let ret = cpu.pc;
			cpu.push_u16(bus, ret);
			cpu.pc = addr;
		}
		0xC4 => {
			let addr = cpu.imm16(bus);
			if !cpu.registers.flag_z() {
				let ret = cpu.pc;
				cpu.push_u16(bus, ret);
				cpu.pc = addr;
			}
		}
		0xCC => {
			let addr = cpu.imm16(bus);
			if cpu.registers.flag_z() {
				let ret = cpu.pc;
				cpu.push_u16(bus, ret);
				cpu.pc = addr;
			}
		}
		0xD4 => {
			let addr = cpu.imm16(bus);
			if !cpu.registers.flag_c() {
				let ret = cpu.pc;
				cpu.push_u16(bus, ret);
				cpu.pc = addr;
			}
		}
		0xDC => {
			let addr = cpu.imm16(bus);
			if cpu.registers.flag_c() {
				let ret = cpu.pc;
				cpu.push_u16(bus, ret);
				cpu.pc = addr;
			}
		}
		0xC9 => cpu.pc = cpu.pop_u16(bus),
		0xD9 => {
			cpu.pc = cpu.pop_u16(bus);
			cpu.ime = true;
		}
		0xC0 => {
			if !cpu.registers.flag_z() {
				cpu.pc = cpu.pop_u16(bus);
			}
		}
		0xC8 => {
			if cpu.registers.flag_z() {
				cpu.pc = cpu.pop_u16(bus);
			}
		}
		0xD0 => {
			if !cpu.registers.flag_c() {
				cpu.pc = cpu.pop_u16(bus);
			}
		}
		0xD8 => {
			if cpu.registers.flag_c() {
				cpu.pc = cpu.pop_u16(bus);
			}
		}
		0xC7 => rst(cpu, bus, 0x00),
		0xCF => rst(cpu, bus, 0x08),
		0xD7 => rst(cpu, bus, 0x10),
		0xDF => rst(cpu, bus, 0x18),
		0xE7 => rst(cpu, bus, 0x20),
		0xEF => rst(cpu, bus, 0x28),
		0xF7 => rst(cpu, bus, 0x30),
		0xFF => rst(cpu, bus, 0x38),

		// ===== CB-prefix =====
		0xCB => {
			let cb = cpu.fetch_byte(bus);
			exec_cb(cpu, bus, cb);
		}

		_ => println!("Opcode {:02X} not implemented (illegal or reserved)", opcode),
	}
}

fn jr(cpu: &mut super::Cpu, offset: i8) {
	cpu.pc = (cpu.pc as i32 + offset as i32) as u16;
}

fn rst(cpu: &mut super::Cpu, bus: &mut MemoryBus, addr: u16) {
	let ret = cpu.pc;
	cpu.push_u16(bus, ret);
	cpu.pc = addr;
}

pub fn exec_cb(cpu: &mut super::Cpu, bus: &mut MemoryBus, opcode: u8) {
	let x = opcode >> 6;
	let y = (opcode >> 3) & 0x07;
	let z = opcode & 0x07;

	match x {
		0 => {
			let value = cpu.read_reg(bus, z);
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
					let cin = if cpu.registers.flag_c() { 1 } else { 0 };
					((value << 1) | cin, c)
				}
				3 => {
					let c = (value & 0x01) != 0;
					let cin = if cpu.registers.flag_c() { 0x80 } else { 0 };
					((value >> 1) | cin, c)
				}
				4 => (value << 1, (value & 0x80) != 0),
				5 => ((value >> 1) | (value & 0x80), (value & 0x01) != 0),
				6 => ((value >> 4) | (value << 4), false),
				7 => (value >> 1, (value & 0x01) != 0),
				_ => unreachable!(),
			};

			cpu.write_reg(bus, z, res);
			cpu.set_flags(res == 0, false, false, carry);
		}
		1 => {
			let value = cpu.read_reg(bus, z);
			let bit_set = (value & (1 << y)) != 0;
			cpu.registers.set_flag_z(!bit_set);
			cpu.registers.set_flag_n(false);
			cpu.registers.set_flag_h(true);
		}
		2 => {
			let value = cpu.read_reg(bus, z);
			cpu.write_reg(bus, z, value & !(1 << y));
		}
		3 => {
			let value = cpu.read_reg(bus, z);
			cpu.write_reg(bus, z, value | (1 << y));
		}
		_ => unreachable!(),
	}
}

