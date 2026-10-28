// Copyright (c) 2024 The xterm.js authors. All rights reserved. MIT license.
//! Browser-independent decisions made by the web-fonts addon.

pub fn unquote(family: &[u16]) -> &[u16] {
    if !family.is_empty() && matches!(family[0], 0x22 | 0x27) && family.last() == family.first() {
        // JS slice(1, -1) is empty even for a one-code-unit quote.
        if family.len() < 2 {
            &[]
        } else {
            &family[1..family.len() - 1]
        }
    } else {
        family
    }
}

fn whitespace(unit: u16) -> bool {
    matches!(unit, 0x0009..=0x000D | 0x0020 | 0x00A0 | 0x1680 | 0x2000..=0x200A | 0x2028 | 0x2029 | 0x202F | 0x205F | 0x3000 | 0xFEFF)
}

pub fn split_family(family: &[u16]) -> Vec<Vec<u16>> {
    if family.is_empty() {
        return Vec::new();
    }
    family
        .split(|unit| *unit == 0x2C)
        .map(|part| {
            let start = part
                .iter()
                .position(|unit| !whitespace(*unit))
                .unwrap_or(part.len());
            let end = part
                .iter()
                .rposition(|unit| !whitespace(*unit))
                .map_or(start, |i| i + 1);
            unquote(&part[start..end]).to_vec()
        })
        .collect()
}

pub fn quote(family: &[u16]) -> Vec<u16> {
    let positive = !family.is_empty()
        && family.iter().all(|unit| matches!(unit, 0x2D | 0x5F | 0x30..=0x39 | 0x41..=0x5A | 0x61..=0x7A | 0xA0..=0xFFFF));
    // The source's negative regex has the multiline flag. Check every JS line.
    let negative = family
        .split(|unit| matches!(unit, 0xA | 0xD | 0x2028 | 0x2029))
        .any(|line| {
            matches!(line.first(), Some(0x30..=0x39))
                || (line.first() == Some(&0x2D) && matches!(line.get(1), Some(0x2D | 0x30..=0x39)))
        });
    if positive && !negative {
        return family.to_vec();
    }
    let mut result = vec![0x22];
    for unit in family {
        if *unit == 0x22 {
            result.push(0x5C);
        }
        result.push(*unit);
    }
    result.push(0x22);
    result
}

pub fn create_family(families: &[Vec<u16>]) -> Vec<u16> {
    let mut result = Vec::new();
    for (index, family) in families.iter().enumerate() {
        if index != 0 {
            result.extend([0x2C, 0x20]);
        }
        result.extend(quote(family));
    }
    result
}

/// Only these five properties participate in upstream FontFace deduplication.
/// Source bytes, display, featureSettings, variationSettings and object identity
/// are deliberately excluded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontDescriptor {
    pub family: Vec<u16>,
    pub stretch: Vec<u16>,
    pub style: Vec<u16>,
    pub unicode_range: Vec<u16>,
    pub weight: Vec<u16>,
}

impl FontDescriptor {
    pub fn normalized(mut self) -> Self {
        self.family = unquote(&self.family).to_vec();
        self
    }
}

/// Compute the two ordered family option writes after all web fonts load.
/// None means no registered web family and therefore no writes or loads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelayoutPlan {
    pub load_families: Vec<Vec<u16>>,
    pub temporary_family: Vec<u16>,
    pub restore_family: Vec<u16>,
}

pub fn relayout_plan(family: &[u16], registered: &[Vec<u16>]) -> Option<RelayoutPlan> {
    let (dirty, clean): (Vec<_>, Vec<_>) = split_family(family)
        .into_iter()
        .partition(|name| registered.iter().any(|value| unquote(value) == name));
    if dirty.is_empty() {
        return None;
    }
    Some(RelayoutPlan {
        load_families: dirty,
        temporary_family: if clean.is_empty() {
            "monospace".encode_utf16().collect()
        } else {
            create_family(&clean)
        },
        restore_family: family.to_vec(),
    })
}
