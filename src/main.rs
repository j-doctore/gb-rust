mod emulator;
mod register;
mod cpu;

use std::env::{self, Args};

use emulator::Emulator;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} /path/to/game", args[0]);
        std::process::exit(1);
    }

    let rom_path = &args[1];


    let emu = Emulator::new();
    emu.load_rom(&rom_path);

}
