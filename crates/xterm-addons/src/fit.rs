// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of addon-fit/src/FitAddon.ts.

//! Sizing uses CSS strings and f64 to preserve JavaScript parseInt/NaN behavior.
//! The browser adapter reads styles from each element's ownerDocument window.

use std::{fmt, rc::Rc};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FitError(pub String);
impl fmt::Display for FitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for FitError {}

pub const DEFAULT_SCROLLBAR_WIDTH: f64 = 14.0;
const MINIMUM_COLS: f64 = 2.0;
const MINIMUM_ROWS: f64 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dimensions {
    pub cols: f64,
    pub rows: f64,
}

#[derive(Clone, Debug)]
pub struct FitMeasurements {
    pub cell_width: f64,
    pub cell_height: f64,
    pub parent_width: String,
    pub parent_height: String,
    /// Top, bottom, right, left, in upstream evaluation order.
    pub padding: [String; 4],
    pub scrollback: f64,
    pub show_scrollbar: Option<bool>,
    pub scrollbar_width: Option<f64>,
}

pub trait FitTerminal {
    /// None means missing element, missing parent or unmeasured dimensions.
    fn measurements(&self) -> Result<Option<FitMeasurements>, FitError>;
    fn resize(&self, dimensions: Dimensions) -> Result<(), FitError>;
}

#[derive(Default)]
pub struct FitAddon {
    terminal: Option<Rc<dyn FitTerminal>>,
}

impl FitAddon {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn activate(&mut self, terminal: Rc<dyn FitTerminal>) {
        self.terminal = Some(terminal);
    }

    /// Upstream dispose deliberately retains the terminal and has no side effects.
    pub fn dispose(&self) {}

    pub fn propose_dimensions(&self) -> Result<Option<Dimensions>, FitError> {
        let Some(terminal) = &self.terminal else {
            return Ok(None);
        };
        Ok(terminal
            .measurements()?
            .as_ref()
            .and_then(propose_dimensions))
    }

    pub fn fit(&self) -> Result<(), FitError> {
        if let (Some(terminal), Some(dimensions)) = (&self.terminal, self.propose_dimensions()?) {
            if !dimensions.cols.is_nan() && !dimensions.rows.is_nan() {
                terminal.resize(dimensions)?;
            }
        }
        Ok(())
    }
}

pub fn propose_dimensions(m: &FitMeasurements) -> Option<Dimensions> {
    if m.cell_width == 0.0 || m.cell_height == 0.0 {
        return None;
    }
    let scrollbar = if m.scrollback == 0.0 || !m.show_scrollbar.unwrap_or(true) {
        0.0
    } else {
        m.scrollbar_width.unwrap_or(DEFAULT_SCROLLBAR_WIDTH)
    };
    let height = parse_css_integer(&m.parent_height).max(0.0);
    let width = parse_css_integer(&m.parent_width).max(0.0);
    let padding = m.padding.each_ref().map(|value| parse_css_integer(value));
    Some(Dimensions {
        cols: js_max(
            MINIMUM_COLS,
            ((width - padding[2] - padding[3] - scrollbar) / m.cell_width).floor(),
        ),
        rows: js_max(
            MINIMUM_ROWS,
            ((height - padding[0] - padding[1]) / m.cell_height).floor(),
        ),
    })
}

fn js_max(a: f64, b: f64) -> f64 {
    if b.is_nan() {
        f64::NAN
    } else {
        a.max(b)
    }
}

/// parseInt(s, 10) || 0, including decimal-prefix truncation and JS whitespace.
pub fn parse_css_integer(value: &str) -> f64 {
    let value = value.trim_start_matches(|c: char| {
        matches!(c, '\u{0009}'..='\u{000d}' | ' ' | '\u{00a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
    });
    let (sign, value) = match value.as_bytes().first() {
        Some(b'-') => (-1.0, &value[1..]),
        Some(b'+') => (1.0, &value[1..]),
        _ => (1.0, value),
    };
    let digits: String = value.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return 0.0;
    }
    // Parsing the complete prefix preserves JS rounding for large decimal values.
    let number = digits
        .parse::<f64>()
        .expect("ASCII decimal digits are a valid floating-point literal")
        * sign;
    if number == 0.0 {
        0.0
    } else {
        number
    }
}
