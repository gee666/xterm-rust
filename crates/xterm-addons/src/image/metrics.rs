// Copyright (c) 2023 The xterm.js authors. MIT license.
//! IIP format sniffing preserves source signed bitwise dimensions, not file validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageType {
    Png,
    Jpeg,
    Gif,
    Qoi,
    Webp,
    Avif,
    Unsupported,
}
impl ImageType {
    pub fn mime(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Gif => "image/gif",
            Self::Qoi => "image/qoi",
            Self::Webp => "image/webp",
            Self::Avif => "image/avif",
            Self::Unsupported => "unsupported",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageMetrics {
    pub mime: ImageType,
    pub width: i32,
    pub height: i32,
}
pub const UNSUPPORTED_TYPE: ImageMetrics = ImageMetrics {
    mime: ImageType::Unsupported,
    width: 0,
    height: 0,
};

fn be32(data: &[u8], offset: usize) -> i32 {
    i32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}
fn le16(data: &[u8], offset: usize) -> i32 {
    i32::from(data[offset]) | (i32::from(data[offset + 1]) << 8)
}
fn le24(data: &[u8], offset: usize) -> i32 {
    le16(data, offset) | (i32::from(data[offset + 2]) << 16)
}

/// Sniff a byte slice. Unlike JS Uint32Array, the Rust slice requires no alignment.
/// Browser bindings must preserve source view-offset rejection at their boundary.
pub fn image_type(data: &[u8]) -> ImageMetrics {
    if data.len() < 32 {
        return UNSUPPORTED_TYPE;
    }
    let (mime, width, height) =
        if data[..8] == [137, 80, 78, 71, 13, 10, 26, 10] && data[12..16] == *b"IHDR" {
            (ImageType::Png, be32(data, 16), be32(data, 20))
        } else if data[..3] == [255, 216, 255] {
            let (width, height) = jpeg_size(data);
            (ImageType::Jpeg, width, height)
        } else if data[..4] == *b"GIF8" && matches!(data[4], b'7' | b'9') && data[5] == b'a' {
            (ImageType::Gif, le16(data, 6), le16(data, 8))
        } else if data[..4] == *b"qoif" {
            (ImageType::Qoi, be32(data, 4), be32(data, 8))
        } else if data[..4] == *b"RIFF" && data[8..12] == *b"WEBP" && data[12..15] == *b"VP8" {
            match data[15] {
                b'X' => (ImageType::Webp, le24(data, 24) + 1, le24(data, 27) + 1),
                b'L' if data[20] == 47 => {
                    let dimensions = u32::from_le_bytes([data[21], data[22], data[23], data[24]]);
                    (
                        ImageType::Webp,
                        (dimensions & 0x3fff) as i32 + 1,
                        ((dimensions >> 14) & 0x3fff) as i32 + 1,
                    )
                }
                b' ' if data[23..26] == [157, 1, 42] => (
                    ImageType::Webp,
                    le16(data, 26) & 0x3fff,
                    le16(data, 28) & 0x3fff,
                ),
                _ => return UNSUPPORTED_TYPE,
            }
        } else if data[4..8] == *b"ftyp" && matches!(&data[8..12], b"avif" | b"avis") {
            let limit = (data.len() - 16).min(1024);
            let Some(position) = (8..limit).find(|&i| data[i..i + 4] == *b"ispe") else {
                return UNSUPPORTED_TYPE;
            };
            let (width, height) = (be32(data, position + 8), be32(data, position + 12));
            if width <= 0 || height <= 0 {
                return UNSUPPORTED_TYPE;
            }
            (ImageType::Avif, width, height)
        } else {
            return UNSUPPORTED_TYPE;
        };
    ImageMetrics {
        mime,
        width,
        height,
    }
}

fn jpeg_size(data: &[u8]) -> (i32, i32) {
    // Typed-array out-of-range reads become zero in the source's bitwise formula.
    let byte = |index| u16::from(data.get(index).copied().unwrap_or(0));
    let mut position = 4;
    let mut block = usize::from((byte(position) << 8) | byte(position + 1));
    loop {
        position += block;
        if position >= data.len() || data[position] != 255 {
            return (0, 0);
        }
        if matches!(data.get(position + 1), Some(192 | 194)) {
            return if position + 8 < data.len() {
                (
                    i32::from((byte(position + 7) << 8) | byte(position + 8)),
                    i32::from((byte(position + 5) << 8) | byte(position + 6)),
                )
            } else {
                (0, 0)
            };
        }
        position += 2;
        block = usize::from((byte(position) << 8) | byte(position + 1));
    }
}
