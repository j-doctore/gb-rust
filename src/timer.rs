const TAC_INC_TIMA_FLAG: u8 = 0b100;

pub struct TimerRegister {
    div: u8,  // FF04 (upper 8 bits of internal divider)
    tima: u8, // FF05
    tma: u8,  // FF06
    tac: u8,  // FF07

    div_counter: u16,
    irq_pending: bool,
}

impl TimerRegister {
    pub fn new() -> TimerRegister {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,

            div_counter: 0,
            irq_pending: false,
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
            0xFF07 => self.set_tac(val & 0x07),
            _ => unreachable!(),
        }
    }

    fn reset_div(&mut self) {
        let old_signal = self.timer_input_signal();

        self.div_counter = 0;
        self.div = 0;

        let new_signal = self.timer_input_signal();
        if old_signal && !new_signal && self.increment_tima() {
            self.irq_pending = true;
        }
    }

    fn selected_div_bit(&self) -> u8 {
        match self.tac & 0x03 {
            0x00 => 9,
            0x01 => 3,
            0x02 => 5,
            0x03 => 7,
            _ => unreachable!(),
        }
    }

    fn timer_enabled(&self) -> bool {
        self.tac & TAC_INC_TIMA_FLAG != 0
    }

    fn timer_input_signal(&self) -> bool {
        if !self.timer_enabled() {
            return false;
        }

        let bit = self.selected_div_bit();
        ((self.div_counter >> bit) & 1) != 0
    }

    fn set_tac(&mut self, new_tac: u8) {
        let old_signal = self.timer_input_signal();
        self.tac = new_tac;
        let new_signal = self.timer_input_signal();

        if old_signal && !new_signal && self.increment_tima() {
            self.irq_pending = true;
        }
    }

    fn increment_tima(&mut self) -> bool {
        let (next_tima, overflowed) = self.tima.overflowing_add(1);
        if overflowed {
            self.tima = self.tma;
            true
        } else {
            self.tima = next_tima;
            false
        }
    }

    fn tick_once(&mut self) -> bool {
        let old_signal = self.timer_input_signal();
        self.div_counter = self.div_counter.wrapping_add(1);
        self.div = (self.div_counter >> 8) as u8;
        let new_signal = self.timer_input_signal();

        old_signal && !new_signal && self.increment_tima()
    }

    pub fn step(&mut self, cycles: u32) -> bool {
        let mut interrupt_requested = self.irq_pending;
        self.irq_pending = false;

        for _ in 0..cycles {
            if self.tick_once() {
                interrupt_requested = true;
            }
        }

        interrupt_requested
    }
}
