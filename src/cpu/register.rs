pub struct Register {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,

    //Flags
    f: u8, //Flags-Bit: Z-7 N-6 H-5 C-4
}

impl Register {
    pub fn new() -> Self {
        Register {
            a: 0x11,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,

            f: 0x80,
        }
    }

    //16-bit pairs
    pub fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | (self.f as u16)
    }
    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = (value as u8) & 0xF0; // low nibble must be 0
    }

    pub fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | (self.c as u16)
    }
    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    pub fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | (self.e as u16)
    }
    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }

    pub fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | (self.l as u16)
    }
    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }

    //8-bit getters/setters
    pub fn get_a(&self) -> u8 {
        self.a
    }
    pub fn set_a(&mut self, v: u8) {
        self.a = v;
    }

    pub fn get_b(&self) -> u8 {
        self.b
    }
    pub fn set_b(&mut self, v: u8) {
        self.b = v;
    }

    pub fn get_c(&self) -> u8 {
        self.c
    }
    pub fn set_c(&mut self, v: u8) {
        self.c = v;
    }

    pub fn get_d(&self) -> u8 {
        self.d
    }
    pub fn set_d(&mut self, v: u8) {
        self.d = v;
    }

    pub fn get_e(&self) -> u8 {
        self.e
    }
    pub fn set_e(&mut self, v: u8) {
        self.e = v;
    }

    pub fn get_h(&self) -> u8 {
        self.h
    }
    pub fn set_h(&mut self, v: u8) {
        self.h = v;
    }

    pub fn get_l(&self) -> u8 {
        self.l
    }
    pub fn set_l(&mut self, v: u8) {
        self.l = v;
    }

    //flag helpers

    //check if flag is active
    pub fn flag_z(&self) -> bool {
        (self.f & 0x80) != 0
    }
    pub fn flag_n(&self) -> bool {
        (self.f & 0x40) != 0
    }
    pub fn flag_h(&self) -> bool {
        (self.f & 0x20) != 0
    }
    pub fn flag_c(&self) -> bool {
        (self.f & 0x10) != 0
    }

    pub fn set_flag_z(&mut self, on: bool) {
        self.f = if on { self.f | 0x80 } else { self.f & !0x80 };
    }
    pub fn set_flag_n(&mut self, on: bool) {
        self.f = if on { self.f | 0x40 } else { self.f & !0x40 };
    }
    pub fn set_flag_h(&mut self, on: bool) {
        self.f = if on { self.f | 0x20 } else { self.f & !0x20 };
    }
    pub fn set_flag_c(&mut self, on: bool) {
        self.f = if on { self.f | 0x10 } else { self.f & !0x10 };
    }
}
