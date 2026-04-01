pub struct Register {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,

    //Flags
    zero: u8,
    subtraction: u8,
    half_carry: u8,
    carry: u8,
}

impl Register {
    pub fn new() -> Self {
        Register {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            zero: 0,
            subtraction: 0,
            half_carry: 0,
            carry: 0,
        }
    }

    pub fn get_af() {
        //TODO
        unimplemented!()
    }

    pub fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | (self.c as u16)
    }

    pub fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | (self.e as u16)
    }

    pub fn get_hl(&self) -> u16 {
        (self.h as u16) >> 8 | (self.l as u16)
    }
}
