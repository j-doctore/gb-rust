use gb_rust::Cartridge; // Re-exported at crate root
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn write_temp_rom(bytes: &[u8]) -> PathBuf {
	let mut path = std::env::temp_dir();
	let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
	path.push(format!("gb_test_rom_{}.gb", now));
	fs::write(&path, bytes).expect("failed to write temp rom");
	path
}

#[test]
fn title_from_header() {
	// Prepare a minimal ROM buffer with header fields and a title at 0x134
	let mut rom = vec![0u8; 0x150];
	// Cartridge type (0x147) = 0x00 (ROM ONLY)
	rom[0x147] = 0x00;
	// ROM size (0x148) = 0x00 -> valid
	rom[0x148] = 0x00;
	// RAM size (0x149) = 0x02 -> 8KB
	rom[0x149] = 0x02;
	let title = b"TESTROM";
	rom[0x134..0x134 + title.len()].copy_from_slice(title);

	let path = write_temp_rom(&rom);
	let cart = Cartridge::new(path.to_str().unwrap()).expect("failed to load cartridge");
	let got = cart.get_title();
	assert!(got.starts_with("TESTROM"), "title did not start with TESTROM: {}", got);
	let _ = fs::remove_file(path);
}

#[test]
fn header_length_and_ram_read_write() {
	// Create ROM with header fields indicating 8KB external RAM
	let mut rom = vec![0u8; 0x150];
	rom[0x147] = 0x00; // ROM ONLY
	rom[0x148] = 0x00; // ROM size valid
	rom[0x149] = 0x02; // 8KB RAM

	let path = write_temp_rom(&rom);
	let mut cart = Cartridge::new(path.to_str().unwrap()).expect("failed to load cartridge");

	// Header (0x100..0x150) should be present
	let header = cart.get_header();
	assert_eq!(header.len(), 0x150 - 0x100);

	// External RAM initial contents should be zero
	assert_eq!(cart.read_ram(0xA000), 0x00);

	// Write and read back inside range
	cart.write_ram(0xA000 + 10, 0xAB);
	assert_eq!(cart.read_ram(0xA000 + 10), 0xAB);

	// Out-of-range reads should return 0xFF
	let beyond = 0xA000 + cart.ram.len() as u16 + 1;
	assert_eq!(cart.read_ram(beyond), 0xFF);

	let _ = fs::remove_file(path);
}


