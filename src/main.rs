mod display;
mod emulator;
mod font;
mod opcodes;
mod program;
mod renderer;

use crate::{
    emulator::EmulatedChip8,
    font::Chip8Font,
    program::Program,
    renderer::{Renderer, TuiRenderer},
};
use clap::Parser;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use spin_sleep::LoopHelper;
use std::{
    path::{Path, PathBuf},
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

    /// The path to log output to
    #[arg(short, long)]
    log_path: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if let Some(log_path) = args.log_path {
        setup_logging(&log_path)?;
    }

    let period_draw = Duration::from_secs_f64(1. / 60.);
    let mut renderer = TuiRenderer::new(period_draw)?;

    let mut emulated_chip8 = EmulatedChip8::new();
    // Load up font and program
    emulated_chip8.write_font(&Chip8Font::new_from_default()?);
    emulated_chip8.load_program(&Program::new_from_file(args.program)?);

    let mut last_draw = Instant::now();
    let mut lh = LoopHelper::builder().build_with_target_rate(args.speed);

    loop {
        lh.loop_start();

        // Check if screen is still alive
        if renderer.terminated() {
            break;
        }

        // Fetch key state
        let key_input = renderer.current_key_state();

        emulated_chip8.step(key_input)?;
        if last_draw.elapsed() > period_draw {
            last_draw = Instant::now();
            renderer.update_screen(&emulated_chip8.get_state().display)?;
        }
        lh.loop_sleep();
    }

    Ok(())
}

fn setup_logging<P: AsRef<Path>>(file: P) -> anyhow::Result<()> {
    let file_appender = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build(file)?;

    let config = Config::builder()
        .appender(Appender::builder().build("file", Box::new(file_appender)))
        .build(Root::builder().appender("file").build(LevelFilter::Info))?;

    log4rs::init_config(config)?;

    Ok(())
}
