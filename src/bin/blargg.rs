use gb_rust::Emulator;


pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} /path/to/testrom.gb", args[0]);
        std::process::exit(1);
    }

    let rom_path = &args[1];

    let mut emu = Emulator::new(&rom_path);
    loop {
        emu.tick();
    }
}
