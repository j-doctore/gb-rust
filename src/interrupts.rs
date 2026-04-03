pub const IF_UNUSED_BITS_MASK: u8 = 0xE0;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InterruptType {
    VBlank,
    LCDSTAT,
    Timer,
    Serial,
    Joypad,
}

impl InterruptType {
    pub fn bit(self) -> u8 {
        match self {
            InterruptType::VBlank => 0,
            InterruptType::LCDSTAT => 1,
            InterruptType::Timer => 2,
            InterruptType::Serial => 3,
            InterruptType::Joypad => 4,
        }
    }

    pub fn mask(self) -> u8 {
        1u8 << self.bit()
    }

    pub fn vector(self) -> u16 {
        match self {
            InterruptType::VBlank => 0x40,
            InterruptType::LCDSTAT => 0x48,
            InterruptType::Timer => 0x50,
            InterruptType::Serial => 0x58,
            InterruptType::Joypad => 0x60,
        }
    }

    pub fn from_bit(bit: u8) -> Option<Self> {
        match bit {
            0 => Some(InterruptType::VBlank),
            1 => Some(InterruptType::LCDSTAT),
            2 => Some(InterruptType::Timer),
            3 => Some(InterruptType::Serial),
            4 => Some(InterruptType::Joypad),
            _ => None,
        }
    }

    pub fn highest_priority_from_pending(pending: u8) -> Option<Self> {
        let bit = pending.trailing_zeros() as u8;
        Self::from_bit(bit)
    }
}

impl From<InterruptType> for usize {
    fn from(irq: InterruptType) -> Self {
        irq.bit() as usize
    }
}