// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Ported from browser/Types.ts.
use super::{channels, Color};

/// The exact default xterm 256-color palette, including the Tango first sixteen.
pub fn default_ansi_colors() -> [Color; 256] {
    const BASE: [u32; 16] = [
        0x2e3436, 0xcc0000, 0x4e9a06, 0xc4a000, 0x3465a4, 0x75507b, 0x06989a, 0xd3d7cf, 0x555753,
        0xef2929, 0x8ae234, 0xfce94f, 0x729fcf, 0xad7fa8, 0x34e2e2, 0xeeeeec,
    ];
    const CUBE: [i64; 6] = [0, 95, 135, 175, 215, 255];
    std::array::from_fn(|index| {
        if index < BASE.len() {
            let packed = BASE[index];
            channels::to_color(
                ((packed >> 16) & 255).into(),
                ((packed >> 8) & 255).into(),
                (packed & 255).into(),
                None,
            )
        } else if index < 232 {
            let index = index - 16;
            channels::to_color(CUBE[index / 36], CUBE[index / 6 % 6], CUBE[index % 6], None)
        } else {
            let gray = 8 + (index as i64 - 232) * 10;
            channels::to_color(gray, gray, gray, None)
        }
    })
}
