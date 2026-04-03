use std::fs;

const CARTRIDGE_TYPE: usize = 0x147;
const ROM_SIZE: usize = 0x148;
const RAM_SIZE: usize = 0x149;
const RAM_BANK_LEN: usize = 8 * 1024;

pub enum CartridgeType {
    RomOnly,
    Mbc1,
    Mbc2,
    Mmm01,
    Mbc3,
    Mbc5,
}
pub struct Cartridge {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    pub rom_banks: u16,
    pub ram_banks: u8,
    pub cart_type: CartridgeType,
}

impl Cartridge {
    pub fn new(path: &str) -> Self {
        let mut cart = Cartridge {
            rom: Vec::new(),
            ram: Vec::new(),
            rom_banks: 0,
            ram_banks: 0,
            cart_type: CartridgeType::RomOnly,
        };
        cart.load(path);
        cart
    }

    fn load(&mut self, path: &str) {
        let rom = fs::read(path).unwrap_or_else(|e| panic!("Failed to read ROM file {}: {}", path, e));
        let cart_type = Self::parse_cartridge_type(&rom)
            .unwrap_or_else(|| panic!("Unsupported cartridge type: {:#04X}", rom[CARTRIDGE_TYPE]));

        let rom_banks = Self::parse_rom_banks(rom[ROM_SIZE])
            .unwrap_or_else(|| panic!("Invalid ROM size header value: {:#04X}", rom[ROM_SIZE]));
        let ram_size = Self::parse_ram_size(rom[RAM_SIZE])
            .unwrap_or_else(|| panic!("Invalid RAM size header value: {:#04X}", rom[RAM_SIZE]));

        self.ram = vec![0; ram_size];
        self.rom = rom;
        self.cart_type = cart_type;
        self.rom_banks = rom_banks;
        self.ram_banks = if ram_size == 0 { 0 } else { (ram_size / RAM_BANK_LEN) as u8 };
    }

    pub fn get_header(&self) -> &[u8] {
        if self.rom.len() <= 0x100 {
            return &[];
        }
        let end = self.rom.len().min(0x150);
        &self.rom[0x100..end]
    }

    pub fn get_title(&self) -> String {
        let start = 0x134;
        let end = 0x144.min(self.rom.len());
        if start >= end {
            return String::new();
        }
        let title_bytes = &self.rom[start..end];
        String::from_utf8_lossy(title_bytes).to_string()
    }

    fn parse_ram_size(header_value: u8) -> Option<usize> {
        let ram_size = match header_value {
            0x00 => 0,
            0x01 => 0,
            0x02 => RAM_BANK_LEN,
            0x03 => 4 * RAM_BANK_LEN,
            0x04 => 16 * RAM_BANK_LEN,
            0x05 => 8 * RAM_BANK_LEN,
            _ => return None,
        };
        Some(ram_size)
    }

    fn parse_rom_banks(header_value: u8) -> Option<u16> {
        let num_banks = match header_value {
            0x00 => 2,
            0x01 => 4,
            0x02 => 8,
            0x03 => 16,
            0x04 => 32,
            0x05 => 64,
            0x06 => 128,
            0x52 => 72,
            0x53 => 80,
            0x54 => 96,
            _ => return None,
        };
        Some(num_banks)
    }

    fn rom_size_from_header(header_value: u8) -> usize {
        match header_value {
            0x00 => 32 * 1024,
            0x01 => 64 * 1024,
            0x02 => 128 * 1024,
            0x03 => 256 * 1024,
            0x04 => 512 * 1024,
            0x05 => 1024 * 1024,
            0x06 => 2048 * 1024,
            0x07 => 4096 * 1024,
            0x08 => 8192 * 1024,
            _ => panic!("Unsupported ROM size header value: {:#04X}", header_value),
        }
    }

    //TODO: refine details (battery, timer, rumble, etc.) once MBCs are implemented
    fn parse_cartridge_type(rom: &[u8]) -> Option<CartridgeType> {
        let cart_type = match rom.get(CARTRIDGE_TYPE)? {
            0x00 => CartridgeType::RomOnly,
            0x01 => CartridgeType::Mbc1,
            0x02 => CartridgeType::Mbc1,
            0x03 => CartridgeType::Mbc1,
            0x05 => CartridgeType::Mbc2,
            0x06 => CartridgeType::Mbc2,
            0x08 => CartridgeType::RomOnly, // ROM ONLY
            0x09 => CartridgeType::RomOnly, // ROM ONLY
            0x0B => CartridgeType::Mmm01,   // MMM01
            0x0C => CartridgeType::Mmm01,   // MMM01
            0x0D => CartridgeType::Mmm01,   // MMM01
            0x0F => CartridgeType::Mbc3,    // MBC3
            0x10 => CartridgeType::Mbc3,    // MBC3
            0x11 => CartridgeType::Mbc5,    // MBC5
            0x12 => CartridgeType::Mbc5,    // MBC5
            0x13 => CartridgeType::Mbc5,    // MBC5
            _ => return None,
        };
        Some(cart_type)
    }
}
