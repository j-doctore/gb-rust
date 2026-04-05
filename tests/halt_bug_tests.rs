use gb_rust::Emulator;
use std::path::PathBuf;

fn rom_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

#[test]
fn halt_bug_blargg_passes() {
    let rom_path = rom_path("test-roms/halt_bug.gb");
    if !rom_path.exists() {
        eprintln!("Skipping halt_bug: ROM not found at {}", rom_path.display());
        return;
    }

    let mut emu = Emulator::new(
        rom_path
            .to_str()
            .expect("ROM path contains invalid UTF-8; cannot run test"),
    )
    .expect("Failed to initialize emulator with halt_bug ROM");

    const MAX_TICKS: usize = 8_000_000;

    for _ in 0..MAX_TICKS {
        emu.tick();

        let out = emu.serial_output();
        if out.contains("Passed") {
            return;
        }

        if out.contains("Failed") {
            panic!("halt_bug reported failure:\n{}", out);
        }

        // Blargg shell fallback: writes exit code A to 0xA000, then loops forever.
        let pc = emu.debug_cpu_pc();
        let op = emu.debug_peek_byte(pc);
        let op_next = emu.debug_peek_byte(pc.wrapping_add(1));
        if op == 0x18 && op_next == 0xFE {
            if let Some((addr, code)) = emu.debug_last_ext_ram_write() {
                if addr == 0xA000 {
                    if code == 0 {
                        return;
                    }
                    panic!(
                        "halt_bug reached exit loop with failure code {} at {:#06X}",
                        code, addr
                    );
                }
            }
        }
    }

    panic!(
        "halt_bug timed out after {} ticks. PC={:#06X} opcode={:#04X}. last_ext_ram_write={:?} code@pc={} text@A004={} Current serial output:\n{}",
        MAX_TICKS,
        emu.debug_cpu_pc(),
        emu.debug_peek_byte(emu.debug_cpu_pc()),
        emu.debug_last_ext_ram_write(),
        (emu.debug_cpu_pc().saturating_sub(8)..=emu.debug_cpu_pc().saturating_add(8))
            .map(|addr| format!("{:02X}", emu.debug_peek_byte(addr)))
            .collect::<Vec<_>>()
            .join(" "),
        (0xA004u16..=0xA040u16)
            .map(|addr| emu.debug_peek_byte(addr))
            .take_while(|b| *b != 0)
            .map(char::from)
            .collect::<String>(),
        emu.serial_output()
    );
}
