const CPU_CLOCK: u8 = 4;
const DIV_CLOCK: u8 = 16;

pub struct TimerRegister {
    div: u8,
    div_clock: u8,

    timer_counter: u8,
    counter_clock: u8,

    timer_modulo: u8,

    timer_control: u8,

    request_interrupt: bool,
}

impl TimerRegister {
    pub fn new() -> TimerRegister {
        Self {
            div: 0,
            div_clock: 0,

            timer_counter: 0,
            counter_clock: 0,

            timer_modulo: 0,
            timer_control: 0,

            request_interrupt: false,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => self.div,
            0xFF05 => self.timer_counter,
            0xFF06 => self.timer_modulo,
            0xFF07 => self.timer_control,
            _ => unreachable!(),
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF04 => self.set_div(val),
            0xFF05 => self.timer_counter = val,
            0xFF06 => self.timer_modulo = val,
            0xFF07 => self.timer_control = val,
            _ => unreachable!(),
        }
    }

    fn set_div(&mut self, _val: u8) {
        self.div_clock = 0;
        self.div = 0;
        //TODO: m-cycle?
    }

    pub fn get_counter_rate(&self) -> Option<u8> {
        //TODO: maybe mask with 0x7 - because of EnableFlag?
        match self.timer_control & 0x3 {
            0x00 => Some(64), // 256/4
            0x01 => Some(1),  // 4/4
            0x02 => Some(4),  //16/4
            0x03 => Some(16), //64/4
            _ => None,
        }
    }

    pub fn inc_div(&mut self) {
        self.div_clock += 1;
        if self.div_clock >= DIV_CLOCK {
            self.div_clock = 0;
            self.div += 1;
        }
    }

    pub fn inc_counter(&mut self, counter_rate: u8) {
        self.counter_clock += 1;

        if self.counter_clock >= counter_rate {
            self.counter_clock = 0;

            if self.timer_counter == 0xFF {
                self.timer_counter = self.timer_modulo;
                self.request_interrupt = true;
            } else {
                self.timer_counter += 1;
            }
        }
    }
}
