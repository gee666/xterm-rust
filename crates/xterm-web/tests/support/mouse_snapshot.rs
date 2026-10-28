//! Controlled values for the read-observer mouse host, not a production pull API.
use xterm_web::input::mouse_policy::MouseRouting;

#[derive(Clone, Copy, Debug)]
pub struct MouseSnapshot {
    pub routing: MouseRouting,
    pub valid_char_size: bool,
    pub css_cell_width: f64,
    pub css_cell_height: f64,
    pub css_canvas_width: f64,
    pub css_canvas_height: f64,
    pub has_scrollback: bool,
    pub application_cursor: bool,
}
