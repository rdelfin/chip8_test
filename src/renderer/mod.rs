use crate::display::Display;
use std::time::Duration;

mod tui;

pub use tui::TuiRenderer;

pub trait Renderer: Sized {
    /// Creates a new renderer of this type. No parameters are provided as this should be created
    /// with whatever defaults we have
    fn new(render_period: Duration) -> anyhow::Result<Self>;

    /// Should return true if the renderer terminates early
    fn terminated(&self) -> bool;

    /// Called every time there's an update to the screen. This being called doesn't necessarily
    /// mean that the data changed, just that we need to render to the screen.
    fn update_screen(&mut self, display: &Display) -> anyhow::Result<()>;
}
