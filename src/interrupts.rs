
pub enum InterruptType {
    VBlank,
    LCDSTAT,
    Timer,
    Joypad,
    Serial
}

impl From<InterruptType> for usize {
    fn from(irq: InterruptType) -> Self {
        match irq {
            InterruptType::VBlank => 0,
            InterruptType::LCDSTAT => 1,
            InterruptType::Timer => 2,
            InterruptType::Serial => 3,
            InterruptType::Joypad => 4,
        }
    }
}

//TODO