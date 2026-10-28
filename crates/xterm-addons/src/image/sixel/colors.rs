// Copyright (c) 2019, 2021 Joerg Breitbart. MIT license.
// Color arithmetic follows node-sixel's actual C/WASM decoder, not CSS HSL.
use super::{DecodeError, SixelDecoder};

const fn rgb(r: i32, g: i32, b: i32) -> u32 {
    let r = r.wrapping_mul(255).wrapping_add(50) / 100;
    let g = g.wrapping_mul(255).wrapping_add(50) / 100;
    let b = b.wrapping_mul(255).wrapping_add(50) / 100;
    0xff00_0000 | (b as u32).wrapping_shl(16) | (g as u32).wrapping_shl(8) | r as u32
}

pub(super) const VT340: [u32; 16] = [
    rgb(0, 0, 0),
    rgb(20, 20, 80),
    rgb(80, 13, 13),
    rgb(20, 80, 20),
    rgb(80, 20, 80),
    rgb(20, 80, 80),
    rgb(80, 80, 20),
    rgb(53, 53, 53),
    rgb(26, 26, 26),
    rgb(33, 33, 60),
    rgb(60, 26, 26),
    rgb(33, 60, 33),
    rgb(60, 33, 60),
    rgb(33, 60, 60),
    rgb(60, 60, 33),
    rgb(80, 80, 80),
];

/// Image addon's VT340-first palette followed by the ANSI cube and grayscale.
pub const ADDON_PALETTE: [u32; 256] = addon_palette();

const fn addon_palette() -> [u32; 256] {
    let mut result = [0; 256];
    let mut i = 0;
    while i < 16 {
        result[i] = VT340[i];
        i += 1;
    }
    let levels = [0_u32, 95, 135, 175, 215, 255];
    let mut r = 0;
    while r < 6 {
        let mut g = 0;
        while g < 6 {
            let mut b = 0;
            while b < 6 {
                result[i] = 0xff00_0000 | levels[b] << 16 | levels[g] << 8 | levels[r];
                i += 1;
                b += 1;
            }
            g += 1;
        }
        r += 1;
    }
    let mut gray = 8;
    while gray <= 238 {
        result[i] = 0xff00_0000 | gray << 16 | gray << 8 | gray;
        gray += 10;
        i += 1;
    }
    result
}

fn hue(t1: f32, t2: f32, mut c: f32) -> f32 {
    if c < 0.0 {
        c += 1.0;
    } else if c > 1.0 {
        c -= 1.0;
    }
    if c < 1.0 / 6.0 {
        t2 + (t1 - t2) * 6.0 * c
    } else if c < 0.5 {
        t1
    } else if c < 2.0 / 3.0 {
        t2 + (t1 - t2) * (4.0 - c * 6.0)
    } else {
        t2
    }
}

fn wasm_channel(value: f32) -> u32 {
    // The shipped WASM checks the unsigned conversion range, then preserves
    // all converted bits. Malformed HLS values can spill into adjacent channels.
    if (0.0..4_294_967_296.0).contains(&value) {
        value as u32
    } else {
        0
    }
}

fn hls(h: i32, l: i32, s: i32) -> u32 {
    if s == 0 {
        return rgb(l, l, l);
    }
    let h = h.wrapping_add(240) as f32 / 360.0;
    let l = l as f32 / 100.0;
    let s = s as f32 / 100.0;
    let t1 = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l * (1.0 - s) + s
    };
    let t2 = l * 2.0 - t1;
    let r = wasm_channel(255.0 * hue(t1, t2, h + 1.0 / 3.0) + 0.5);
    let g = wasm_channel(255.0 * hue(t1, t2, h) + 0.5);
    let b = wasm_channel(255.0 * hue(t1, t2, h - 1.0 / 3.0) + 0.5);
    0xff00_0000 | b.wrapping_shl(16) | g.wrapping_shl(8) | r
}

impl SixelDecoder {
    pub(super) fn apply_color(&mut self) -> Result<(), DecodeError> {
        // Parentheses preserve the C ternary's surprising precedence. In the
        // HLS branch the original checks hue, but not luminance or saturation.
        let valid = if self.param_len == 5 && self.params[1] == 1 {
            self.params[2] <= 360
        } else {
            self.params[2] <= 100 && self.params[3] <= 100 && self.params[4] <= 100
        };
        if self.param_len != 1 && !valid {
            return Ok(());
        }
        if self.palette_limit == 0 {
            return Err(DecodeError::PaletteZero);
        }
        let index = (self.params[0] as u32 % self.palette_limit) as usize;
        if self.param_len != 1 && self.params[1] != 0 && self.params[1] < 3 {
            self.palette[index] = match self.params[1] {
                1 => hls(self.params[2], self.params[3], self.params[4]),
                2 => rgb(self.params[2], self.params[3], self.params[4]),
                _ => return Err(DecodeError::InvalidColor),
            };
        }
        self.color = self.palette[index];
        Ok(())
    }
}
