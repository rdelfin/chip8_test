mod display;
mod emulator;
mod font;
mod opcodes;
mod program;

use crate::{emulator::EmulatedChip8, font::Chip8Font, program::Program};
use clap::Parser;
use std::path::PathBuf;

/// A chip 8 emulator, running with a GUI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the program to load
    #[arg(short, long)]
    program: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut emulated_chip8 = EmulatedChip8::new();
    // Load up font and program
    emulated_chip8.write_font(&Chip8Font::new_from_default()?);
    emulated_chip8.load_program(&Program::new_from_file(args.program)?);

    emulated_chip8.step()?;
    let state = emulated_chip8.get_state();
    println!("{state}");
    Ok(())
}
