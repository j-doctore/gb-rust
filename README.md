# Gameboy Classic Emulator written in Rust

A Gameboy Classic (DMG-01) emulator written in Rust which I am building for educational purposes.

## Features:

## minor Achievements:
* Blargg: all CPU_instructions except Interrupts working
* Controls should be working, test later once Screen is visible


## TODOs
* INTERRUPTS
* clocking/cycling
* PPU/ GRAPHICS
* Banking
* Halt Bug, OAM-DMA, Blargg tests
* Sound support
* MBCs
* refactor

## Controls:
| Button  | Keyboard |
| ------------- | ------------- |
| A  | Right  |
| B  | Left  |
| Start  | Return  |
| Select  | Backspace  |
| Left  | A  |
| Right  | D  |
| Up  | W  |
| Down  | S  |

## Goals
* simple graphics/maybe UI
* be close to Hardware emulation
* pass many Blargg-Tests (for DMG, no CGB), maybe some from Mooneye
* Sound-Support?
* real Games: get at least Tetris and Pokemon working

## Limitations | What will not be covered
* no Serial Data support
* no GB-Camera/Printer functionality
* maybe not all MBCs will be supported

## License

This project is for educational purposes. Feel free to use, modify, and learn from it.
