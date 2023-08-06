mod display;
mod emulator;
mod font;
mod opcodes;
mod program;

use crate::{emulator::EmulatedChip8, font::Chip8Font, program::Program};
use clap::Parser;
use spin_sleep::LoopHelper;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

/// A chip 8 emulator, running with a GUI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the program to load
    #[arg(short, long)]
    program: PathBuf,

    /// The speed at which the processor runs, in Hz
    #[arg(short, long, default_value_t = 500.)]
    speed: f64,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut emulated_chip8 = EmulatedChip8::new();
    // Load up font and program
    emulated_chip8.write_font(&Chip8Font::new_from_default()?);
    emulated_chip8.load_program(&Program::new_from_file(args.program)?);

    let period_draw = Duration::from_secs_f64(1. / 60.);
    let mut last_draw = Instant::now();
    let mut lh = LoopHelper::builder().build_with_target_rate(args.speed);

    loop {
        lh.loop_start();
        emulated_chip8.step()?;
        if last_draw.elapsed() > period_draw {
            last_draw = Instant::now();
            println!("{}", emulated_chip8.get_state().display);
        }
        lh.loop_sleep();
    }
}
