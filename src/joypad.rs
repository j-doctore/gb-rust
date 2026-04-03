// Minimal Joypad implementation for JOYP (FF00)
// Only implements select bits + released buttons behavior.

pub struct Joypad {
    // Only keep bits 4 and 5 (P14/P15). High bits read as 1.
    select: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad { select: 0x30 }
    }

    // Read JOYP: bits 7-6 = 1, bits 5-4 = select, bits 3-0 = active-low buttons (released = 1)
    pub fn read(&self) -> u8 {
        0xC0 | (self.select & 0x30) | 0x0F
    }

    // Write to JOYP: only bits 4 and 5 have effect
    pub fn write(&mut self, value: u8) {
        self.select = value & 0x30;
    }
}
