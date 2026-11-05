// Copyright (c) 2021 The xterm.js authors. All rights reserved. MIT license.
use super::{
    arc::svg_arc,
    types::{Area, DrawCommand as C, GlyphCanvas},
};

#[derive(Default)]
struct State {
    current: [f64; 2],
    control: [f64; 2],
    last: char,
}

#[derive(Clone, Copy)]
pub(super) enum PathMode {
    Direct,
    Snapped,
    Vector { left: f64, right: f64, dpr: f64 },
    Clip,
    Negative,
}

fn parse(token: &str) -> f64 {
    // Source paths include glued close commands such as `.27Z`. parseFloat
    // consumes the numeric prefix, unlike Rust's whole-string parser.
    let token = token.trim_start();
    let bytes = token.as_bytes();
    let mut end = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    if token.get(end..).is_some_and(|s| s.starts_with("Infinity")) {
        return if bytes.first() == Some(&b'-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }
    let start = end;
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    let mut digits = end - start;
    if bytes.get(end) == Some(&b'.') {
        end += 1;
        let start = end;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        digits += end - start;
    }
    if digits == 0 {
        return f64::NAN;
    }
    if matches!(bytes.get(end), Some(b'e' | b'E')) {
        let exponent = end;
        end += 1;
        if matches!(bytes.get(end), Some(b'+' | b'-')) {
            end += 1;
        }
        let start = end;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        if end == start {
            end = exponent;
        }
    }
    token[..end].parse().unwrap_or(f64::NAN)
}

fn snap(value: f64) -> f64 {
    if value == 0.0 {
        value
    } else {
        crate::renderer::utils::js_round(value + 0.5) - 0.5
    }
}

pub(super) fn draw_path<T: GlyphCanvas>(
    canvas: &mut T,
    path: &str,
    area: Area,
    mode: PathMode,
) -> Result<(), T::Error> {
    let mut state = State::default();
    for instruction in path.split(' ') {
        let Some(kind) = instruction.chars().next() else {
            continue;
        };
        if kind == 'Z' {
            canvas.draw(C::Close)?;
            state.last = kind;
            continue;
        }
        if matches!(mode, PathMode::Snapped | PathMode::Vector { .. })
            && !matches!(kind, 'M' | 'L' | 'C' | 'Q' | 'T')
        {
            canvas.draw(C::LogError(format!(
                "Could not find drawing instructions for \"{kind}\""
            )))?;
            continue;
        }
        let values: Vec<_> = instruction[1..].split(',').collect();
        if matches!(mode, PathMode::Direct) {
            if kind == 'H' || kind == 'V' {
                let mut point = state.current;
                if kind == 'H' {
                    point[0] = area.x + parse(values[0]) * area.width;
                } else {
                    point[1] = area.y + parse(values[0]) * area.height;
                }
                canvas.draw(C::Line(point))?;
                state.current = point;
                state.control = point;
                state.last = kind;
                continue;
            }
            if kind == 'A' && values.len() >= 7 {
                let p: Vec<_> = values.iter().map(|s| parse(s)).collect();
                let end = [area.x + p[5] * area.width, area.y + p[6] * area.height];
                svg_arc(
                    canvas,
                    state.current,
                    end,
                    [p[0] * area.width, p[1] * area.height],
                    p[2] * std::f64::consts::PI / 180.0,
                    p[3],
                    p[4],
                )?;
                state.current = end;
                // The source A branch deliberately leaves last/control untouched.
                continue;
            }
        }
        if values.len() < 2 || values[0].is_empty() || values[1].is_empty() {
            continue;
        }
        let (left, right, dpr) = match mode {
            PathMode::Vector { left, right, dpr } => (left, right, dpr),
            _ => (0.0, 0.0, 1.0),
        };
        let translated: Vec<_> = values
            .iter()
            .enumerate()
            .map(|(i, value)| {
                if i % 2 == 0 {
                    let scaled = parse(value) * (area.width - left * dpr - right * dpr);
                    area.x
                        + left * dpr
                        + if matches!(mode, PathMode::Snapped) {
                            snap(scaled)
                        } else {
                            scaled
                        }
                } else {
                    let scaled = parse(value) * area.height;
                    area.y
                        + if matches!(mode, PathMode::Snapped) {
                            snap(scaled)
                        } else {
                            scaled
                        }
                }
            })
            .collect();
        let p = [translated[0], translated[1]];
        match kind {
            'M' | 'L' => {
                canvas.draw(if kind == 'M' { C::Move(p) } else { C::Line(p) })?;
                state.current = p;
                state.control = p;
            }
            _ if matches!(mode, PathMode::Clip | PathMode::Negative) => {}
            'C' if translated.len() >= 6 => {
                canvas.draw(C::Cubic([
                    p[0],
                    p[1],
                    translated[2],
                    translated[3],
                    translated[4],
                    translated[5],
                ]))?;
                state.control = [translated[2], translated[3]];
                state.current = [translated[4], translated[5]];
            }
            'Q' if translated.len() >= 4 => {
                canvas.draw(C::Quadratic([p[0], p[1], translated[2], translated[3]]))?;
                state.control = p;
                state.current = [translated[2], translated[3]];
            }
            'T' => {
                let control = if matches!(state.last, 'Q' | 'T') {
                    [
                        2.0 * state.current[0] - state.control[0],
                        2.0 * state.current[1] - state.control[1],
                    ]
                } else {
                    state.current
                };
                canvas.draw(C::Quadratic([control[0], control[1], p[0], p[1]]))?;
                state.control = control;
                state.current = p;
            }
            _ => {}
        }
        state.last = kind;
    }
    Ok(())
}
