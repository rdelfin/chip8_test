use crate::{display::Display, renderer::Renderer};
use anyhow::Context;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::Color,
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Rectangle},
        Block, Borders,
    },
    Frame, Terminal,
};
use spin_sleep::LoopHelper;
use std::{
    io::Stdout,
    sync::{
        mpsc::{self, Receiver, Sender, TryRecvError},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

type CrossTerminal = Terminal<CrosstermBackend<Stdout>>;

pub struct TuiRenderer {
    terminal: Arc<Mutex<CrossTerminal>>,
    jh: Option<JoinHandle<anyhow::Result<()>>>,
    display: Arc<Mutex<Display>>,
    stop_tx: Sender<()>,
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

        let (stop_tx, stop_rx) = mpsc::channel();

        let display: Arc<Mutex<Display>> = Arc::default();
        let display_clone = display.clone();

        Ok(TuiRenderer {
            terminal,
            jh: Some(thread::spawn(move || {
                Self::run_loop(terminal_clone, display_clone, render_period, stop_rx)
            })),
            display,
            stop_tx,
        })
    }

    fn terminated(&self) -> bool {
        self.jh.as_ref().map(|jh| jh.is_finished()).unwrap_or(true)
    }

    fn update_screen(&mut self, display: &Display) -> anyhow::Result<()> {
        *self.display.lock().unwrap() = display.clone();
        Ok(())
    }
}

impl TuiRenderer {
    fn run_loop(
        terminal: Arc<Mutex<CrossTerminal>>,
        display: Arc<Mutex<Display>>,
        render_period: Duration,
        stop_rx: Receiver<()>,
    ) -> anyhow::Result<()> {
        let poll_timeout = render_period / 4;
        let mut lh = LoopHelper::builder().build_with_target_rate(1. / render_period.as_secs_f32());
        loop {
            lh.loop_start();
            // If we got a message or the other side disconnected, stop the loop
            if matches!(stop_rx.try_recv(), Ok(_) | Err(TryRecvError::Disconnected)) {
                return Ok(());
            }
            {
                let display = display.lock().unwrap();
                let mut terminal = terminal.lock().unwrap();
                terminal.draw(|frame| Self::draw(frame, &display))?
            };
            if Self::should_quit(poll_timeout)? {
                break;
            }
            lh.loop_sleep();
        }
        Ok(())
    }

    fn draw(f: &mut Frame<'_>, display: &Display) {
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

        let canvas = Canvas::default()
            .block(
                Block::default()
                    .title("Chip 8 Display")
                    .borders(Borders::ALL),
            )
            .marker(Marker::Block)
            .paint(|ctx| {
                for y in 0..display.pixels.len() {
                    for x in 0..display.pixels[y].len() {
                        if display.pixels[y][x] {
                            ctx.draw(&Rectangle {
                                x: x as f64,
                                y: -(y as f64),
                                width: 1.,
                                height: 1.,
                                color: Color::White,
                            })
                        }
                    }
                }
            })
            .x_bounds([0.0, 64.0])
            .y_bounds([-32.0, 0.0]);
        f.render_widget(canvas, chunks[1]);
    }

    fn should_quit(timeout: Duration) -> anyhow::Result<bool> {
        if event::poll(timeout).context("event poll failed")? {
            if let Event::Key(key) = event::read().context("event read failed")? {
                return Ok(KeyCode::Char('q') == key.code);
            }
        }
        Ok(false)
    }

    fn reset_terminal() -> anyhow::Result<()> {
        disable_raw_mode().context("failed to disable raw mode")?;
        execute!(std::io::stdout(), LeaveAlternateScreen)
            .context("unable to switch to main screen")?;
        Ok(())
    }
}

impl Drop for TuiRenderer {
    fn drop(&mut self) {
        // We can ignore failures as the `jh.join()` call below will propagate errors in the run
        // loop
        let _ = self.stop_tx.send(());

        if let Some(jh) = self.jh.take() {
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
