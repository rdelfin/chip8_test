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
use log::{debug, error, info, LevelFilter};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use spin_sleep::LoopHelper;
use std::{
    any::Any,
    backtrace::Backtrace,
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

    /// The speed at which the processor runs, in Hz.
    /// Default is 700 instructions/second as a rough average of real timing
    #[arg(short, long, default_value_t = 700.)]
    speed: f64,

    /// The path to log output to
    #[arg(short, long)]
    log_path: Option<PathBuf>,

    /// Enables verbose logging (logs debug logs too)
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if let Some(log_path) = args.log_path {
        setup_logging(log_path, args.verbose)?;
    }

    std::panic::set_hook(Box::new(move |panic_info| {
        let backtrace = Backtrace::capture();
        let payload = payload_as_str(panic_info.payload());
        if let Some(location) = panic_info.location() {
            error!(
                "panic occurred in file '{}' at line {}: {payload}",
                location.file(),
                location.line(),
            );
        } else {
            error!("panic occured: {payload}");
        }
        error!("backtrace: {backtrace}");
        std::process::exit(1);
    }));

    let period_draw = Duration::from_secs_f64(1. / 60.);
    let mut renderer = TuiRenderer::new(period_draw)?;

    let mut emulated_chip8 = EmulatedChip8::new();
    // Load up font and program
    emulated_chip8.write_font(&Chip8Font::new_from_default()?);
    emulated_chip8.load_program(&Program::new_from_file(args.program)?);

    let mut last_draw = Instant::now();
    let mut lh = LoopHelper::builder().build_with_target_rate(args.speed);
    let expected_period = Duration::from_secs_f64(1. / args.speed);

    loop {
        lh.loop_start();

        // Check if screen is still alive
        if renderer.terminated() {
            info!("terminating program");
            debug!("final state:\n{}", emulated_chip8.get_state());
            break;
        }

        // Fetch key state
        let key_input = renderer.current_key_state();

        emulated_chip8.step(key_input, expected_period)?;
        if last_draw.elapsed() > period_draw {
            last_draw = Instant::now();
            renderer.update_screen(&emulated_chip8.get_state().display)?;
        }
        lh.loop_sleep();
    }

    Ok(())
}

fn payload_as_str(payload: &dyn Any) -> &str {
    if let Some(&s) = payload.downcast_ref::<&'static str>() {
        s
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.as_str()
    } else {
        "Box<dyn Any>"
    }
}

fn setup_logging<P: AsRef<Path>>(file: P, verbose: bool) -> anyhow::Result<()> {
    let file_appender = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build(file)?;

    let config = Config::builder()
        .appender(Appender::builder().build("file", Box::new(file_appender)))
        .build(Root::builder().appender("file").build(if verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        }))?;

    log4rs::init_config(config)?;

    Ok(())
}
