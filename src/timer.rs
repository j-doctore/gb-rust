pub struct TimerRegister {
    div: u8,          // FF04
    tima: u8,         // FF05
    tma: u8,          // FF06
    tac: u8,          // FF07

    div_counter: u16,
    tima_counter: u16,
}

impl TimerRegister {
    pub fn new() -> TimerRegister {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,

            div_counter: 0,
            tima_counter: 0,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => self.div,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac | 0xF8,
            _ => unreachable!(),
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF04 => self.reset_div(),
            0xFF05 => self.tima = val,
            0xFF06 => self.tma = val,
            0xFF07 => self.tac = val & 0x07,
            _ => unreachable!(),
        }
    }

    fn reset_div(&mut self) {
        self.div_counter = 0;
        self.div = 0;
    }

    fn timer_period(&self) -> u16 {
        match self.tac & 0x03 {
            0x00 => 1024, //~ 4096 Hz: 4_000_000/4096 = 1024 cycles
            0x01 => 16,
            0x02 => 64,
            0x03 => 256,
            _ => unreachable!(),
        }
    }

    pub fn step(&mut self, t_cycles: u32) -> bool {
        let mut request_timer_interrupt = false;

        self.div_counter = self.div_counter.wrapping_add(t_cycles as u16);
        while self.div_counter >= 256 {
            self.div_counter -= 256;
            self.div = self.div.wrapping_add(1);
        }

        if (self.tac & 0x04) != 0 {
            let period = self.timer_period();
            self.tima_counter = self.tima_counter.wrapping_add(t_cycles as u16);

            while self.tima_counter >= period {
                self.tima_counter -= period;
                let (next, overflow) = self.tima.overflowing_add(1);
                if overflow {
                    self.tima = self.tma;
                    request_timer_interrupt = true;
                } else {
                    self.tima = next;
                }
            }
        }

        request_timer_interrupt
    }
}
