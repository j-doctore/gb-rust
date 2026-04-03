mod bus;
mod cartridge;
mod cpu;
mod emulator;
mod ppu;
mod register;
mod timer;
mod interrupts;

use sdl2::{event::Event, keyboard::Keycode, pixels::Color, render::Canvas, video::Window, rect::Rect};
use std::env::{self};

use emulator::Emulator;
const SCALING: u32 = 5;
const SCREEN_WIDTH: u32 = 160;
const SCREEN_HEIGHT: u32 = 144;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} /path/to/game", args[0]);
        std::process::exit(1);
    }

    let mut emu = Emulator::new(&args[1]);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(&args[1], SCREEN_WIDTH * SCALING, SCREEN_HEIGHT * SCALING)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
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
        emu.tick();
        draw_screen(&emu, &mut canvas)
    }
}

//TODO: display screen, as well as Tiles and sprite data
fn draw_screen(emulator: &Emulator, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.present();
}
