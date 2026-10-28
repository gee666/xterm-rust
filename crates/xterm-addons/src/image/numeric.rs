// Copyright (c) 2020-2023 The xterm.js authors. MIT license.
//! Exact toFixed precisions used by image protocol replies, not Rust tie-to-even.
pub fn fixed_zero(value: f64) -> String {
    fixed(value, 1, 0)
}
pub fn fixed_three(value: f64) -> String {
    fixed(value, 1000, 3)
}

fn fixed(value: f64, scale: u128, digits: usize) -> String {
    let magnitude = value.abs();
    if !value.is_finite() || magnitude >= 1e21 {
        return ryu_js::Buffer::new().format(value).to_owned();
    }
    let bits = magnitude.to_bits();
    let biased = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1_u64 << 52) - 1);
    let (mantissa, exponent) = if biased == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1_u64 << 52), biased - 1023 - 52)
    };
    let scaled = u128::from(mantissa) * scale;
    let rounded = if exponent >= 0 {
        // The only callers use scale1 or1000, and values below1e21. The exact
        // numerator therefore fits80bits and the shift cannot overflow u128.
        scaled << exponent
    } else {
        let shift = exponent.unsigned_abs();
        if shift >= u128::BITS {
            0
        } else {
            let quotient = scaled >> shift;
            let remainder = scaled & ((1_u128 << shift) - 1);
            quotient + u128::from(remainder >= (1_u128 << (shift - 1)))
        }
    };
    let sign = if value < 0.0 { "-" } else { "" };
    if digits == 0 {
        return format!("{sign}{rounded}");
    }
    let integer = rounded / scale;
    let fraction = rounded % scale;
    format!("{sign}{integer}.{fraction:0width$}", width = digits)
}

// Copyright (c) 2026 The xterm.js authors. KittyGraphicsTypes parseInt radix10.
fn whitespace(unit: u16) -> bool {
    matches!(unit, 0x0009..=0x000d | 0x0020 | 0x00a0 | 0x1680 | 0x2000..=0x200a |
        0x2028 | 0x2029 | 0x202f | 0x205f | 0x3000 | 0xfeff)
}

pub(crate) fn parse_decimal_prefix(value: &[u16]) -> f64 {
    let start = value
        .iter()
        .position(|unit| !whitespace(*unit))
        .unwrap_or(value.len());
    let mut end = start;
    if matches!(value.get(end), Some(43 | 45)) {
        end += 1;
    }
    let digits = end;
    while value.get(end).is_some_and(|unit| (48..=57).contains(unit)) {
        end += 1;
    }
    if end == digits {
        return f64::NAN;
    }
    // Accumulating directly in f64 rounds at every digit and differs from JS
    // parseInt above 2^53. Parse the validated ASCII decimal once instead. This
    // nonempty digit prefix with an optional ASCII sign is valid float syntax;
    // overflow yields infinity, and a negative zero sign remains observable.
    let decimal: String = value[start..end]
        .iter()
        .map(|unit| char::from(*unit as u8))
        .collect();
    decimal
        .parse()
        .expect("validated signed ASCII integer is valid binary64 syntax")
}
