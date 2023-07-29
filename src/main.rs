mod display;
mod emulator;

use crate::emulator::EmulatedChip8;

fn main() {
    let emulated_chip8 = EmulatedChip8::new();
    println!("{emulated_chip8:?}")
}
