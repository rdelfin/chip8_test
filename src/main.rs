mod display;
mod emulator;
mod font;
mod opcodes;

use crate::emulator::EmulatedChip8;

fn main() -> anyhow::Result<()> {
    let mut emulated_chip8 = EmulatedChip8::new();
    emulated_chip8.step()?;
    let state = emulated_chip8.get_state();
    println!("{state}");
    Ok(())
}
