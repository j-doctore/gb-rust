use crate::cpu::register::Register;
use crate::membus::MemoryBus;



pub fn add_a(reg: &mut Register, value: u8, carry_in: u8) {
    let a = reg.get_a();
    let (tmp, c1) = a.overflowing_add(value);
    let (res, c2) = tmp.overflowing_add(carry_in);

    let half = ((a & 0x0F) + (value & 0x0F) + carry_in) > 0x0F;
    let carry = c1 || c2;

    reg.set_a(res);
    reg.set_flag_z(res == 0);
    reg.set_flag_n(false);
    reg.set_flag_h(half);
    reg.set_flag_c(carry);
}

pub fn sub_a(reg: &mut Register, value: u8, carry_in: u8) {
    let a = reg.get_a();
    let (tmp, b1) = a.overflowing_sub(value);
    let (res, b2) = tmp.overflowing_sub(carry_in);

    let half = (a & 0x0F) < ((value & 0x0F) + carry_in);
    let carry = b1 || b2;

    reg.set_a(res);
    reg.set_flag_z(res == 0);
    reg.set_flag_n(true);
    reg.set_flag_h(half);
    reg.set_flag_c(carry);
}

pub fn and_a(reg: &mut Register, value: u8) {
    let res = reg.get_a() & value;
    reg.set_a(res);
    reg.set_flag_z(res == 0);
    reg.set_flag_n(false);
    reg.set_flag_h(true);
    reg.set_flag_c(false);
}

pub fn xor_a(reg: &mut Register, value: u8) {
    let res = reg.get_a() ^ value;
    reg.set_a(res);
    reg.set_flag_z(res == 0);
    reg.set_flag_n(false);
    reg.set_flag_h(false);
    reg.set_flag_c(false);
}

pub fn or_a(reg: &mut Register, value: u8) {
    let res = reg.get_a() | value;
    reg.set_a(res);
    reg.set_flag_z(res == 0);
    reg.set_flag_n(false);
    reg.set_flag_h(false);
    reg.set_flag_c(false);
}

pub fn cp_a(reg: &mut Register, value: u8) {
    let a = reg.get_a();
    let res = a.wrapping_sub(value);
    let half = (a & 0x0F) < (value & 0x0F);
    let carry = a < value;

    reg.set_flag_z(res == 0);
    reg.set_flag_n(true);
    reg.set_flag_h(half);
    reg.set_flag_c(carry);
}

pub fn add_hl(reg: &mut Register, value: u16) {
    let hl = reg.get_hl();
    let res = hl.wrapping_add(value);
    let half = ((hl & 0x0FFF) + (value & 0x0FFF)) > 0x0FFF;
    let carry = (hl as u32 + value as u32) > 0xFFFF;
    reg.set_hl(res);
    reg.set_flag_n(false);
    reg.set_flag_h(half);
    reg.set_flag_c(carry);
}

pub fn daa(reg: &mut Register) {
    let mut a = reg.get_a();
    let n = reg.flag_n();
    let mut c = reg.flag_c();
    let h = reg.flag_h();

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

    reg.set_a(a);
    reg.set_flag_z(a == 0);
    reg.set_flag_h(false);
    reg.set_flag_c(c);
}

pub fn alu(cpu: &mut super::Cpu, bus: &mut MemoryBus, y: u8, z: u8) {
    let value = cpu.read_reg(bus, z);
    let carry_in = if cpu.registers.flag_c() { 1 } else { 0 };
    match y {
        0 => add_a(&mut cpu.registers, value, 0),
        1 => add_a(&mut cpu.registers, value, carry_in),
        2 => sub_a(&mut cpu.registers, value, 0),
        3 => sub_a(&mut cpu.registers, value, carry_in),
        4 => and_a(&mut cpu.registers, value),
        5 => xor_a(&mut cpu.registers, value),
        6 => or_a(&mut cpu.registers, value),
        7 => cp_a(&mut cpu.registers, value),
        _ => unreachable!(),
    }
}
