pub mod components;
pub mod layout;
pub mod theme;

use ratatui::Frame;
use crate::app::App;
use self::layout::render_ui;

pub fn draw(f: &mut Frame, app: &App) {
    render_ui(f, app);
}
