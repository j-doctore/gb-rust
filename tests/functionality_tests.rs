use gb_rust::Cartridge; // Re-exported at crate root


#[test]
fn get_cart_title() {
    //todo!();
    let rom_path = "test-roms/halt_bug.gb";
    let mut cart = Cartridge::new();
    cart.load(rom_path);
    println!("Title: {}", cart.get_title());
}

