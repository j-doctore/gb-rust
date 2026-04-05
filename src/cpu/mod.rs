use crate::interrupts::InterruptType;
use crate::membus::MemoryBus;
mod alu;
mod microops;
mod opcodes;
mod register;
use self::register::Register;

pub struct Cpu {
    registers: Register,
    pc: u16,
    sp: u16,
    ime: bool,        //Interrupt Master Enable
    ei_delay: u8, //EI delayed by one instruction, we need to track if pending
    halted: bool,
    halt_bug: bool,
    step_cycles: u32,
    stepped_cycles: u32,
}

impl Cpu {
    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn a(&self) -> u8 {
        self.registers.get_a()
    }

    pub fn new() -> Self {
        Self {
            registers: Register::new(),
            pc: 0x0100,
            sp: 0xFFFE,
            ime: false,
            ei_delay: 0,
            halted: false,
            halt_bug: false,
            step_cycles: 0,
            stepped_cycles: 0,
        }
    }

    fn read_byte(&mut self, bus: &mut MemoryBus, addr: u16) -> u8 {
        let value = bus.read_byte(addr);
        let cycles = microops::bus_access();
        self.step_cycles = self.step_cycles.saturating_add(cycles);
        bus.step(cycles);
        self.stepped_cycles = self.stepped_cycles.saturating_add(cycles);
        value
    }

    fn write_byte(&mut self, bus: &mut MemoryBus, addr: u16, val: u8) {
        let cycles = microops::bus_access();
        self.step_cycles = self.step_cycles.saturating_add(cycles);
        bus.write_byte(addr, val);
        bus.step(cycles);
        self.stepped_cycles = self.stepped_cycles.saturating_add(cycles);
    }

    fn fetch_byte(&mut self, bus: &mut MemoryBus) -> u8 {
        let byte = self.read_byte(bus, self.pc);

        self.pc = self.pc.wrapping_add(1);

        byte
    }

    fn fetch_opcode(&mut self, bus: &mut MemoryBus) -> u8 {
        let byte = self.read_byte(bus, self.pc);
        if self.halt_bug {
            // HALT bug: next opcode is fetched without incrementing PC once.
            self.halt_bug = false;
        } else {
            self.pc = self.pc.wrapping_add(1);
        }
        byte
    }

    fn enter_halt(&mut self, bus: &MemoryBus) {
        let pending = bus.read_byte(0xFFFF) & bus.read_byte(0xFF0F) & 0x1F;

        if self.ime {
            self.halted = true;
            self.halt_bug = false;
            return;
        }

        if pending != 0 {
            // IME=0 and pending interrupt triggers HALT bug.
            self.halted = false;
            self.halt_bug = true;
        } else {
            self.halted = true;
            self.halt_bug = false;
        }
    }

    fn add_internal_m_cycles(&mut self, m_cycles: u32) {
        let cycles = microops::m_cycles(m_cycles);
        self.step_cycles = self.step_cycles.saturating_add(cycles);
    }

    fn finalize_step_cycles(&mut self, bus: &mut MemoryBus) {
        if self.step_cycles > self.stepped_cycles {
            let remaining = self.step_cycles - self.stepped_cycles;
            bus.step(remaining);
            self.stepped_cycles = self.step_cycles;
        }
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
        self.stepped_cycles = 0;

        let ie = bus.read_byte(0xFFFF);
        let if_f = bus.read_byte(0xFF0F);
        let pending = ie & if_f;

        if self.halted {
            if pending != 0 {
                self.halted = false;
            } else {
                self.step_cycles = microops::halt_idle();
                bus.step(self.step_cycles);
                self.stepped_cycles = self.step_cycles;
                return self.step_cycles;
            }
        }

        if pending != 0 && self.ime {
            if let Some(i_req) = InterruptType::highest_priority_from_pending(pending) {
                self.service_interrupt(bus, i_req);
                self.finalize_step_cycles(bus);
                return self.step_cycles;
            }
        }

    let opcode = self.fetch_opcode(bus);
        crate::cpu::opcodes::execute(self, bus, opcode);

        if self.ei_delay > 0 {
            self.ei_delay -= 1;
            if self.ei_delay == 0 {
                self.ime = true;
            }
        }

        self.finalize_step_cycles(bus);

        self.step_cycles
    }

    fn service_interrupt(&mut self, bus: &mut MemoryBus, interrupt: InterruptType) {
        self.ime = false;
        self.push_u16(bus, self.pc);
        let current_if_f = self.read_byte(bus, 0xFF0F);
        self.write_byte(bus, 0xFF0F, current_if_f & !interrupt.mask());
        self.pc = interrupt.vector();
        // Interrupt service costs 5 M-cycles total; push/read/write account for 4, add 1 internal.
        self.add_internal_m_cycles(1);
    }
}