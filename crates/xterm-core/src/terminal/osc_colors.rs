// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! Source OSC color requests. Palette storage belongs to the rendering host.
use crate::input::xparse_color::parse_color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorIndex {
    Indexed(u8),
    Foreground,
    Background,
    Cursor,
}
impl ColorIndex {
    pub fn protocol_index(self) -> u16 {
        match self {
            Self::Indexed(index) => u16::from(index),
            Self::Foreground => 256,
            Self::Background => 257,
            Self::Cursor => 258,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColorRequest {
    Report { index: ColorIndex },
    Set { index: ColorIndex, color: [u8; 3] },
    Restore { index: Option<ColorIndex> },
}

pub(super) fn requests(ident: u16, data: &[u16]) -> Vec<Vec<ColorRequest>> {
    let slots: Vec<_> = data.split(|unit| *unit == u16::from(b';')).collect();
    match ident {
        4 => {
            let batch: Vec<_> = slots
                .chunks_exact(2)
                .filter_map(|pair| request(ColorIndex::Indexed(index(pair[0])?), pair[1]))
                .collect();
            if batch.is_empty() {
                Vec::new()
            } else {
                vec![batch]
            }
        }
        10..=12 => {
            let special = [
                ColorIndex::Foreground,
                ColorIndex::Background,
                ColorIndex::Cursor,
            ];
            special[usize::from(ident - 10)..]
                .iter()
                .zip(slots)
                .filter_map(|(&index, spec)| request(index, spec).map(|request| vec![request]))
                .collect()
        }
        104 => {
            if data.is_empty() {
                return vec![vec![ColorRequest::Restore { index: None }]];
            }
            let batch: Vec<_> = slots
                .into_iter()
                .filter_map(|slot| {
                    index(slot).map(|index| ColorRequest::Restore {
                        index: Some(ColorIndex::Indexed(index)),
                    })
                })
                .collect();
            if batch.is_empty() {
                Vec::new()
            } else {
                vec![batch]
            }
        }
        110..=112 => {
            let special = [
                ColorIndex::Foreground,
                ColorIndex::Background,
                ColorIndex::Cursor,
            ];
            vec![vec![ColorRequest::Restore {
                index: Some(special[usize::from(ident - 110)]),
            }]]
        }
        _ => Vec::new(),
    }
}

fn index(data: &[u16]) -> Option<u8> {
    if data.is_empty() {
        return None;
    }
    let mut value = 0_u16;
    for &digit in data {
        if !(u16::from(b'0')..=u16::from(b'9')).contains(&digit) {
            return None;
        }
        value = value * 10 + digit - u16::from(b'0');
        if value > 255 {
            return None;
        }
    }
    Some(value as u8)
}

fn request(index: ColorIndex, spec: &[u16]) -> Option<ColorRequest> {
    if spec == [u16::from(b'?')] {
        return Some(ColorRequest::Report { index });
    }
    // Accepted X11 syntax is ASCII. This also rejects malformed UTF-16 without
    // lossy replacement or changing valid surrounding color characters.
    let bytes: Option<Vec<_>> = spec.iter().map(|&unit| u8::try_from(unit).ok()).collect();
    let text = String::from_utf8(bytes?).ok()?;
    parse_color(&text).map(|color| ColorRequest::Set { index, color })
}
