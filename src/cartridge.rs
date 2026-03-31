const CARTRIDGE_TYPE : usize = 0x147;
const ROM_SIZE : usize = 0x148;
const RAM_SIZE : usize = 0x149;


pub struct Cartridge {
    pub data: Vec<u8>,
}

impl Cartridge {
    pub fn new() -> Self {
        Cartridge { data: Vec::new() }
    }

    pub fn load(&mut self, path: &str) {
        self.data = std::fs::read(path).expect("Failed to read ROM file");
    }

    pub fn get_header(&self) -> &[u8] {
        &self.data[0x100..0x150]
    }

    pub fn get_title(&self) -> String {
        let title_bytes = &self.data[0x134..0x144];
        String::from_utf8_lossy(title_bytes).to_string()
    }
}