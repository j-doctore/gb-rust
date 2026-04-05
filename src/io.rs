use crate::joypad::Joypad;
use crate::interrupts::{INTERRUPT_UNUSED_BITS_MASK, InterruptType};
use crate::joypad::UserInput;
use crate::timer::TimerRegister;


const IO_SIZE: usize = 0x80; // FF00-FF7F (128 bytes)

pub struct IoRegisters {
    joypad: Joypad,
    timers: TimerRegister,
    
    if_reg: u8,
    io: [u8; IO_SIZE],
    serial_out: String,
}

impl IoRegisters {
    pub fn new() -> Self {
        Self {
            joypad: Joypad::new(),
            if_reg: 0xE1,
            timers: TimerRegister::new(),
            io: [0; IO_SIZE],
            serial_out: String::new(),
        }
    }

    pub fn step(&mut self, cycles: u32) {
        if self.timers.step(cycles) {
            self.request_interrupt(InterruptType::Timer);
        }
    }

    pub fn read_io_reg(&self, addr: u16) -> u8 {
        match addr & 0x00FF {
            0x00 => self.joypad.read(),
            0x0F => self.if_reg | INTERRUPT_UNUSED_BITS_MASK,
            0x04..=0x07 => self.timers.read_byte(addr),
            //TODO
            _ => self.io[addr as usize - 0xFF00],
        }
    }

    pub fn write_io(&mut self, addr: u16, value: u8) {
        match addr & 0x00FF {
            0x00 => self.joypad.write(value),
            0x0F => self.if_reg = value & 0x1F,
            0x04..=0x07 => self.timers.write_byte(addr, value),
            //TODO
            _ => {
                self.io[addr as usize - 0xFF00] = value;
                //DEBUG BLARGG:
                if addr == 0xFF02 && value == 0x81 {
                    let c = self.io[0x01] as char;
                    self.serial_out.push(c);
                    print!("{}", c);
                }
            }
        }
    }

    pub fn serial_output(&self) -> &str {
        &self.serial_out
    }

    pub fn press_input(&mut self, input: UserInput) {
        self.joypad.press_button(input);
    }

    pub fn release_input(&mut self, input: UserInput) {
        self.joypad.release_button(input);
    }

    pub fn request_interrupt(&mut self, interrupt: InterruptType) {
        self.if_reg |= interrupt.mask();
    }
}