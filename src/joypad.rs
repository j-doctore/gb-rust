// Minimal Joypad implementation for now
// button pressed indicatedby 0, released by 1 (= active-low)

const SELECT_MASK: u8 = 0b110000; // Only bits 4 and 5 are used for select
const SELECT_BUTTONS: u8 = 0b100000;
const SELECT_DPAD: u8 = 0b10000;
const INPUT_MASK: u8 = 0x0F; // Only bits 0-3 are used for input

#[derive(Debug, Copy, Clone)]
pub enum UserInput {
    A,
    B,
    Start,
    Select,
    Up,
    Down,
    Left,
    Right,
}
pub struct Joypad {
    select: u8,
    keys: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            select: SELECT_MASK,
            keys: 0xFF,
        }
    }

    // Write to JOYP: only bits 4 and 5 have effect
    pub fn write(&mut self, value: u8) {
        self.select = value & SELECT_MASK;
    }

    pub fn set_buttons(&mut self, input: u8) {
        self.keys = (self.keys & INPUT_MASK) | ((input & INPUT_MASK) << 4);
    }

    pub fn set_dpad(&mut self, input: u8) {
        self.keys = (self.keys & 0xF0) | (input & INPUT_MASK);
    }

    pub fn read(&self) -> u8 {
        let sel = self.select & SELECT_MASK;
        let dpad = self.keys & INPUT_MASK;
        let buttons = (self.keys >> 4) & INPUT_MASK;

        let lower = match ((sel & SELECT_DPAD) == 0, (sel & SELECT_BUTTONS) == 0) {
            (true, true) => dpad & buttons,
            (true, false) => dpad,
            (false, true) => buttons,
            (false, false) => INPUT_MASK,
        };

        0xC0 | sel | (lower & INPUT_MASK)
    }

    fn key_line_bit(input: UserInput) -> (bool, u8) {
        match input {
            UserInput::Right => (false, 1 << 0),
            UserInput::Left => (false, 1 << 1),
            UserInput::Up => (false, 1 << 2),
            UserInput::Down => (false, 1 << 3),
            UserInput::A => (true, 1 << 0),
            UserInput::B => (true, 1 << 1),
            UserInput::Select => (true, 1 << 2),
            UserInput::Start => (true, 1 << 3),
        }
    }

    pub fn press_button(&mut self, input: UserInput) {
        let (is_buttons, bit) = Self::key_line_bit(input);

        if is_buttons {
            //Buttons
            let mut line = (self.keys >> 4) & INPUT_MASK;
            line &= !bit;
            self.set_buttons(line);
        } else {
            //Directions
            let mut line = self.keys & INPUT_MASK;
            line &= !bit;
            self.set_dpad(line);
        }
        println!("Pressed {:?}, joypad state: {:08b}", input, self.keys);
    }

    pub fn release_button(&mut self, input: UserInput) {
        let (is_buttons, bit) = Self::key_line_bit(input);

        if is_buttons {
            let mut line = (self.keys >> 4) & INPUT_MASK;
            line |= bit;
            self.set_buttons(line);
        } else {
            let mut line = self.keys & INPUT_MASK;
            line |= bit;
            self.set_dpad(line);
        }
        println!("Released {:?}, joypad state: {:08b}", input, self.keys);
    }
}
