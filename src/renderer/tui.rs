use crate::{display::Display, emulator::KeyInput, renderer::Renderer};
use anyhow::Context;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::info;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use spin_sleep::LoopHelper;
use std::{
    io::Stdout,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

type CrossTerminal = Terminal<CrosstermBackend<Stdout>>;

pub struct TuiRenderer {
    terminal: Arc<Mutex<CrossTerminal>>,
    render_jh: Option<JoinHandle<anyhow::Result<()>>>,
    event_jh: Option<JoinHandle<anyhow::Result<()>>>,
    key_state: Arc<Mutex<(KeyInput, [Instant; 0x10])>>,
    display: Arc<Mutex<Display>>,
    stop_state: Arc<AtomicBool>,
}

impl Renderer for TuiRenderer {
    fn new(render_period: Duration) -> anyhow::Result<TuiRenderer> {
        let mut stdout = std::io::stdout();
        enable_raw_mode().context("failed to enable raw mode")?;
        execute!(stdout, EnterAlternateScreen).context("unable to enter alternate screen")?;

        // Setup panic handler to cleanup terminal
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic| {
            Self::reset_terminal().unwrap();
            original_hook(panic);
        }));

        let terminal = Arc::new(Mutex::new(
            Terminal::new(CrosstermBackend::new(stdout)).context("creating terminal failed")?,
        ));
        let terminal_clone = terminal.clone();

        // let (stop_tx, stop_rx) = mpsc::channel();
        let stop_state: Arc<AtomicBool> = Arc::default();
        let stop_state_clone = stop_state.clone();
        let stop_state_clone_2 = stop_state.clone();

        let display: Arc<Mutex<Display>> = Arc::default();
        let display_clone = display.clone();

        let key_state: Arc<Mutex<(KeyInput, [Instant; 0x10])>> =
            Arc::new(Mutex::new((KeyInput::default(), [Instant::now(); 0x10])));
        let key_state_clone = key_state.clone();

        Ok(TuiRenderer {
            terminal,
            render_jh: Some(thread::spawn(move || {
                Self::run_loop(
                    terminal_clone,
                    display_clone,
                    render_period,
                    stop_state_clone,
                )
            })),
            event_jh: Some(thread::spawn(move || {
                Self::event_loop(key_state_clone, stop_state_clone_2)
            })),
            display,
            stop_state,
            key_state,
        })
    }

    fn terminated(&self) -> bool {
        join_handle_finished(&self.event_jh) || join_handle_finished(&self.render_jh)
    }

    fn current_key_state(&self) -> KeyInput {
        self.key_state.lock().unwrap().0.clone()
    }

    fn update_screen(&mut self, display: &Display) -> anyhow::Result<()> {
        *self.display.lock().unwrap() = display.clone();
        Ok(())
    }
}

fn join_handle_finished<T>(jh: &Option<JoinHandle<T>>) -> bool {
    jh.as_ref().map(|jh| jh.is_finished()).unwrap_or(true)
}

impl TuiRenderer {
    const KEY_PRESS_DURATION: Duration = Duration::from_millis(500);

    fn event_loop(
        key_state: Arc<Mutex<(KeyInput, [Instant; 0x10])>>,
        stop_state: Arc<AtomicBool>,
    ) -> anyhow::Result<()> {
        const POLL_TIMEOUT: Duration = Duration::from_millis(100);

        loop {
            if stop_state.load(Ordering::Relaxed) {
                break;
            }

            // Clear out key states over the duration, since we don't get key up events
            {
                let mut lg = key_state.lock().unwrap();
                for i in 0..lg.0.key_state.len() {
                    if lg.0.key_state[i] && lg.1[i].elapsed() > Self::KEY_PRESS_DURATION {
                        lg.0.key_state[i] = false;
                    }
                }
            }

            if event::poll(POLL_TIMEOUT).context("event poll failed")? {
                if let Event::Key(key) = event::read().context("event read failed")? {
                    let mut keypad_val = None;

                    match key.code {
                        KeyCode::Esc => {
                            info!("Got request to exit (esc pressed)");
                            stop_state.store(true, Ordering::Relaxed);
                            break;
                        }
                        KeyCode::Char('1') => keypad_val = Some(0x1),
                        KeyCode::Char('2') => keypad_val = Some(0x2),
                        KeyCode::Char('3') => keypad_val = Some(0x3),
                        KeyCode::Char('4') => keypad_val = Some(0xC),
                        KeyCode::Char('q') => keypad_val = Some(0x4),
                        KeyCode::Char('w') => keypad_val = Some(0x5),
                        KeyCode::Char('e') => keypad_val = Some(0x6),
                        KeyCode::Char('r') => keypad_val = Some(0xD),
                        KeyCode::Char('a') => keypad_val = Some(0x7),
                        KeyCode::Char('s') => keypad_val = Some(0x8),
                        KeyCode::Char('d') => keypad_val = Some(0x9),
                        KeyCode::Char('f') => keypad_val = Some(0xE),
                        KeyCode::Char('z') => keypad_val = Some(0xA),
                        KeyCode::Char('x') => keypad_val = Some(0x0),
                        KeyCode::Char('c') => keypad_val = Some(0xB),
                        KeyCode::Char('v') => keypad_val = Some(0xF),
                        _ => {}
                    }

                    if let Some(keypad_val) = keypad_val {
                        if key.kind == KeyEventKind::Press {
                            info!("Keypad button {:#x} pressed", keypad_val);
                            let mut lg = key_state.lock().unwrap();
                            lg.1[keypad_val] = Instant::now();
                            lg.0.key_state[keypad_val] = true;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn run_loop(
        terminal: Arc<Mutex<CrossTerminal>>,
        display: Arc<Mutex<Display>>,
        render_period: Duration,
        stop_state: Arc<AtomicBool>,
    ) -> anyhow::Result<()> {
        let mut lh = LoopHelper::builder().build_with_target_rate(1. / render_period.as_secs_f32());
        loop {
            lh.loop_start();
            // Check if the loop was stopped
            if stop_state.load(Ordering::Relaxed) {
                return Ok(());
            }
            {
                let display = display.lock().unwrap();
                let mut terminal = terminal.lock().unwrap();
                terminal.draw(|frame| Self::draw(frame, &display))?
            };
            lh.loop_sleep();
        }
    }

    fn draw(f: &mut Frame<'_>, display: &Display) {
        let display_str = display_to_str(display);

        let size = f.size();

        let chunks = Layout::default()
            .constraints(
                [
                    Constraint::Min(1),
                    Constraint::Ratio(2, 1),
                    Constraint::Min(1),
                ]
                .as_ref(),
            )
            .split(size);

        let canvas = Paragraph::new(display_str).block(
            Block::default()
                .title("Chip 8 Display")
                .borders(Borders::ALL),
        );
        f.render_widget(canvas, chunks[1]);
    }

    fn reset_terminal() -> anyhow::Result<()> {
        disable_raw_mode().context("failed to disable raw mode")?;
        execute!(std::io::stdout(), LeaveAlternateScreen)
            .context("unable to switch to main screen")?;
        Ok(())
    }
}

fn display_to_str(display: &Display) -> String {
    let mut display_str = String::new();
    // Every char will encode two vertical pixels, so we step by 2 in y
    for y_idx in (0..display.pixels.len()).step_by(2) {
        for x_idx in 0..display.pixels[y_idx].len() {
            display_str += match (
                display.pixels[y_idx][x_idx],
                display.pixels[y_idx + 1][x_idx],
            ) {
                (false, false) => " ",
                (true, false) => "▀",
                (false, true) => "▄",
                (true, true) => "█",
            };
        }
        display_str += "\n";
    }
    display_str
}

impl Drop for TuiRenderer {
    fn drop(&mut self) {
        // We can ignore failures as the `jh.join()` call below will propagate errors in the run
        // loop
        self.stop_state.store(true, Ordering::Relaxed);

        if let Some(jh) = self.render_jh.take() {
            jh.join().unwrap().unwrap();
        }
        if let Some(jh) = self.event_jh.take() {
            jh.join().unwrap().unwrap();
        }
        let mut terminal = self.terminal.lock().unwrap();
        Self::reset_terminal().unwrap();
        terminal
            .show_cursor()
            .context("unable to show cursor")
            .unwrap();
    }
}
