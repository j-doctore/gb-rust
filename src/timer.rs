const TAC_INC_TIMA_FLAG: u8 = 0b100;

pub enum Frequency {
    ///~ 4096 Hz: 4_000_000/4096 = 1024 cycles
    F4096 = 1024,
    ///~ 262144 Hz: 4_000_000/262144 = 16 cycles
    F262144 = 16,
    ///~ 65536 Hz: 4_000_000/65536 = 64 cycles
    F65536 = 64,
    /// ~ 16384 Hz: 4_000_000/16384 = 256 cycles
    F16384 = 256,
}

impl Frequency {
    /// The number of CPU cycles that occur per tick of the clock.
    /// = equal to #CPU-cycles per second (4194304 ~ 4.19 MHz) divided by timer frequency.
    fn cycles_per_tick(self) -> usize {
        self as usize
    }
}

pub struct TimerRegister {
    div: u8,  // FF04
    tima: u8, // FF05
    tma: u8,  // FF06
    tac: u8,  // FF07

    frequency: Frequency,
    div_counter: u32,
    tima_counter: u32,
}

impl TimerRegister {
    pub fn new() -> TimerRegister {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,

            frequency: Frequency::F4096,
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

    fn clock_frequency(&self) -> u16 {
        match self.tac & 0x03 {
            0x00 => 1024, //~ 4096 Hz: 4_000_000/4096 = 1024 cycles
            0x01 => 16,
            0x02 => 64,
            0x03 => 256,
            _ => unreachable!(),
        }
    }

    fn tick_div(&mut self) {
        self.div = self.div.wrapping_add(1);
        self.div_counter += 1;

        //TODO: tick at ~16Mhz
        /*if self.div_counter >= 16Mhz
        self.div_counter = 0
        */
    }

    //tima_counter unused; is that right?
    fn tick_tima(&mut self) -> bool{
        let freguency = self.clock_frequency();
        let mut interrupt_requested = false;
        //Only increment TIMA if Enable is set
        if self.tac & TAC_INC_TIMA_FLAG != 0 {
            self.tima += 1
        }

        //TODO: tick at frequency specified by TAC
        // proper overflow logic - i think >= is not sufficient;
        if self.tima >= 0xFF {
            self.tima = self.tma;
            interrupt_requested = true;
        }
        interrupt_requested
    }

    //is this sufficient?
    //cycles unused?
    pub fn step(&mut self, cycles: u32) -> bool {
        self.tick_div();
        self.tick_tima()
    }
}
