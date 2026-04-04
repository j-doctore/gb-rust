use crate::membus::MemoryBus;
pub mod register;
pub mod microops;
pub mod alu;
pub mod opcodes;
use crate::cpu::register::Register;

pub struct Cpu {
    registers: Register,
    pc: u16,
    sp: u16,
    ime: bool,        //Interrupt Master Enable
    ei_pending: bool, //EI delayed by one instruction, we need to track if pending
    halted: bool,
    step_cycles: u32,
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
        }
    }

    fn read_byte(&mut self, bus: &mut MemoryBus, addr: u16) -> u8 {
        bus.read_byte(addr)
    }

    fn write_byte(&mut self, bus: &mut MemoryBus, addr: u16, val: u8) {
        bus.write_byte(addr, val);
    }

    fn fetch_byte(&mut self, bus: &mut MemoryBus) -> u8 {
        let byte = self.read_byte(bus, self.pc);

        self.pc = self.pc.wrapping_add(1);

        byte
    }

    fn push_u8(&mut self, bus: &mut MemoryBus, val: u8) {
        self.sp = self.sp.wrapping_sub(1);
        self.write_byte(bus, self.sp, val);
    }

    fn pop_u8(&mut self, bus: &mut MemoryBus) -> u8 {
        let val = self.read_byte(bus, self.sp);
        self.sp = self.sp.wrapping_add(1);
        val
    }

    fn push_u16(&mut self, bus: &mut MemoryBus, val: u16) {
        let hi = (val >> 8) as u8;
        let lo = val as u8;
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

    pub fn read_reg(&mut self, bus: &mut MemoryBus, reg_index: u8) -> u8 {
        match reg_index {
            0 => self.registers.get_b(),
            1 => self.registers.get_c(),
            2 => self.registers.get_d(),
            3 => self.registers.get_e(),
            4 => self.registers.get_h(),
            5 => self.registers.get_l(),
            6 => self.read_byte(bus, self.registers.get_hl()),
            7 => self.registers.get_a(),
            _ => {
                eprintln!("read_r: invalid register index {} - returning 0", reg_index);
                0
            }
        }
    }

    pub fn write_reg(&mut self, bus: &mut MemoryBus, reg_index: u8, data: u8) {
        match reg_index {
            0 => self.registers.set_b(data),
            1 => self.registers.set_c(data),
            2 => self.registers.set_d(data),
            3 => self.registers.set_e(data),
            4 => self.registers.set_h(data),
            5 => self.registers.set_l(data),
            6 => self.write_byte(bus, self.registers.get_hl(), data),
            7 => self.registers.set_a(data),
            _ => eprintln!(
                "write_r: invalid register index {} - write ignored",
                reg_index
            ),
        }
    }

    //TODO: interrupts, halting, and add timers
    pub fn step(&mut self, bus: &mut MemoryBus) -> u32 {
        self.step_cycles = 0;

        if self.halted {
                return self.step_cycles;
        }

        let opcode = self.fetch_byte(bus);
        crate::cpu::opcodes::execute(self, bus, opcode);

        self.step_cycles
    }

}
