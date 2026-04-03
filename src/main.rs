mod bus;
mod cartridge;
mod cpu;
mod emulator;
mod ppu;
mod register;
mod timer;
mod interrupts;
mod joypad;

use sdl2::{event::Event, keyboard::Keycode, pixels::Color, render::Canvas, video::Window, rect::Rect};
use std::env::{self};

use emulator::Emulator;
const SCALING: u32 = 5;
const SCREEN_WIDTH: u32 = 160;
const SCREEN_HEIGHT: u32 = 144;
const CYCLES_PER_FRAME: u32 = 70_224;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} /path/to/game", args[0]);
        std::process::exit(1);
    }

    let mut emu = match Emulator::new(&args[1]) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Failed to initialize emulator: {}", err);
            std::process::exit(1);
        }
    };

    let sdl_context = match sdl2::init() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("SDL initialization failed: {}", e);
            std::process::exit(1);
        }
    };

    let video_subsystem = match sdl_context.video() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("SDL video subsystem init failed: {}", e);
            std::process::exit(1);
        }
    };

    let window = match video_subsystem
        .window(&args[1], SCREEN_WIDTH * SCALING, SCREEN_HEIGHT * SCALING)
        .position_centered()
        .build()
    {
        Ok(w) => w,
        Err(e) => {
            eprintln!("SDL window creation failed: {}", e);
            std::process::exit(1);
        }
    };

    let mut canvas = match window.into_canvas().present_vsync().build() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("SDL canvas creation failed: {}", e);
            std::process::exit(1);
        }
    };
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        emu.run_cycles(CYCLES_PER_FRAME);
        draw_screen(&emu, &mut canvas)
    }
}

//TODO: display screen, as well as Tiles and sprite data
fn draw_screen(emulator: &Emulator, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let frame_buffer = emulator.get_display();
    for (y, row) in frame_buffer.iter().enumerate() {
        for (x, shade) in row.iter().enumerate() {
            let (r, g, b) = shade_to_rgb(*shade);
            canvas.set_draw_color(Color::RGB(r, g, b));
            let _ = canvas.fill_rect(Rect::new(
                (x as i32) * SCALING as i32,
                (y as i32) * SCALING as i32,
                SCALING,
                SCALING,
            ));
        }
    }

    canvas.present();
}

fn shade_to_rgb(s: u8) -> (u8, u8, u8) {
    match s & 0x03 {
           0 => (255, 255, 255), // white
           1 => (192, 192, 192), // light gray
           2 => (96, 96, 96),     // dark gray
           _ => (0, 0, 0),       // black
    }
}
