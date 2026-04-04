use std::fs;

const CARTRIDGE_TYPE: usize = 0x147;
const INDEX_ROM_SIZE: usize = 0x148;
const INDEX_RAM_SIZE: usize = 0x149;
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
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_banks: u16,
    ram_banks: u8,
    cart_type: CartridgeType,
}

impl Cartridge {
    pub fn new(path: &str) -> Result<Self, String> {
        let data = Self::load(path)?;
        let (rom_size, rom_banks) = Self::parse_rom_size_and_banks(&data)
            .ok_or_else(|| "Invalid ROM size in cartridge header".to_string())?;
        let (ram_size, ram_banks) = Self::parse_ram_size_and_banks(&data)
            .ok_or_else(|| "Invalid RAM size in cartridge header".to_string())?;
        let cart_type = Self::parse_cartridge_type(&data)
            .ok_or_else(|| "Unsupported cartridge type".to_string())?;
        let cart = Cartridge {
            rom: data,
            ram: vec![0; ram_size],
            rom_banks,
            ram_banks,
            cart_type: CartridgeType::RomOnly,
        };
        Ok(cart)
    }

    fn load(path: &str) -> Result<Vec<u8>, String> {
        fs::read(path).map_err(|e| format!("Failed to read ROM file {}: {}", path, e))
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

    pub fn get_ram_size(&self) -> usize {
        self.ram.len()
    }

    fn parse_ram_size_and_banks(data: &[u8]) -> Option<(usize, u8)> {
        let ram_size = match data[INDEX_RAM_SIZE] {
            0x00 => (0, 0),
            0x01 => (0, 0),
            0x02 => (RAM_BANK_LEN, 1),
            0x03 => (4 * RAM_BANK_LEN, 4),
            0x04 => (16 * RAM_BANK_LEN, 16),
            0x05 => (8 * RAM_BANK_LEN, 8),
            _ => return None,
        };
        Some(ram_size)
    }

    fn parse_rom_size_and_banks(data: &[u8]) -> Option<(usize, u16)> {
        let num_banks = match data[INDEX_ROM_SIZE] {
            0x00 => (32 * 1024, 2),
            0x01 => (64 * 1024, 4),
            0x02 => (128 * 1024, 8),
            0x03 => (256 * 1024, 16),
            0x04 => (512 * 1024, 32),
            0x05 => (1024 * 1024, 64),
            0x06 => (2048 * 1024, 128),
            0x52 => (72 * 1024, 72),
            0x53 => (80 * 1024, 80),
            0x54 => (96 * 1024, 96),
            _ => return None,
        };
        Some(num_banks)
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

    pub fn read_rom(&self, addr: u16) -> u8 {
        if (addr as usize) < self.rom.len() {
            return self.rom[addr as usize];
        }
        0xFF
    }

    /// Read external RAM (0xA000..0xBFFF). Return 0xFF otherwise
    pub fn read_ext_ram(&self, addr: u16) -> u8 {
        let idx = addr as usize - 0xA000;
        if idx < self.ram.len() {
            self.ram[idx]
        } else {
            0xFF
        }
    }

    /// Write to external RAM (0xA000..0xBFFF). Ignore otherwise.
    pub fn write_ram(&mut self, addr: u16, value: u8) {
        let idx = addr as usize - 0xA000;
        if idx < self.ram.len() {
            self.ram[idx] = value;
        }
    }
}
