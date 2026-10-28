// Cmap decoding follows opentype.js. See licenses/opentype-js-MIT.txt.
use super::{
    font_bytes::{FontError, Reader},
    font_tables::Glyph,
};
#[path = "font_mac_tables.rs"]
mod mac;

#[derive(Clone, Debug)]
struct Segment {
    start: u16,
    end: u16,
    delta: i16,
    glyphs: Option<Vec<u16>>,
}
#[derive(Clone, Debug)]
struct Group {
    start: u32,
    end: u32,
    glyph: u32,
}
#[derive(Clone, Debug)]
enum Mapping {
    Bytes {
        glyphs: Vec<u8>,
        decoding: &'static [u16],
    },
    Segments(Vec<Segment>),
    Groups {
        groups: Vec<Group>,
        incremental: bool,
    },
}
#[derive(Clone, Debug)]
pub struct Cmap(Mapping);

impl Cmap {
    pub fn parse(bytes: &[u8]) -> Result<Self, FontError> {
        let reader = Reader::new(bytes);
        if reader.u16(0)? != 0 {
            return Err(FontError::Invalid("cmap table version should be 0."));
        }
        let count = usize::from(reader.u16(2)?);
        reader.bytes(4, count * 8)?;
        let mut offset = None;
        let mut platform = 0;
        let mut encoding = 0;
        let mut variation = None;
        for index in (0..count).rev() {
            platform = reader.u16(4 + index * 8)?;
            encoding = reader.u16(6 + index * 8)?;
            let unicode = (platform == 3 && [0, 1, 10].contains(&encoding))
                || (platform == 0 && [0, 1, 2, 3, 4, 6].contains(&encoding))
                || (platform == 1 && encoding == 0);
            if unicode {
                if offset.is_some_and(|offset| offset > 0) {
                    continue;
                }
                offset = Some(reader.offset32(8 + index * 8)?);
                if variation.is_some() {
                    break;
                }
            } else if platform == 0 && encoding == 5 {
                let table = reader.at(reader.offset32(8 + index * 8)?)?;
                variation = if table.u16(0)? == 14 {
                    Some(table)
                } else {
                    None
                };
                if variation.is_some() && offset.is_some_and(|offset| offset > 0) {
                    break;
                }
            }
        }
        let table =
            reader.at(offset.ok_or(FontError::Invalid("No valid cmap sub-tables found."))?)?;
        let format = table.u16(0)?;
        if let Some(variation) = variation {
            validate_variations(variation)?;
        }
        Ok(Self(match format {
            0 => {
                let count = usize::from(table.u16(2)?);
                let language = i32::from(table.u16(4)?) - 1;
                let decoding = mac::mac_encoding(platform, encoding, language)
                    .ok_or(FontError::Invalid("Unsupported cmap format0 encoding"))?;
                // The reference reads length bytes after its six-byte header,
                // even though the declared length includes that header.
                let input = table.bytes(6, count)?;
                let mut glyphs = Vec::new();
                glyphs.try_reserve_exact(count)?;
                glyphs.extend_from_slice(input);
                Mapping::Bytes { glyphs, decoding }
            }
            4 => Mapping::Segments(parse_segments(table)?),
            12 | 13 => Mapping::Groups {
                groups: parse_groups(table)?,
                incremental: format == 12,
            },
            _ => return Err(FontError::UnsupportedFormat("cmap", format)),
        }))
    }

    pub fn glyph(&self, codepoint: u32) -> Glyph {
        match &self.0 {
            Mapping::Bytes { glyphs, decoding } => {
                // Source first copies every byte-index key, then overwrites
                // non-ASCII character keys through the selected Mac decoder.
                if let Some(index) = decoding
                    .iter()
                    .rposition(|unit| u32::from(*unit) == codepoint)
                {
                    return glyphs.get(index + 128).copied().map_or(0, Glyph::from);
                }
                usize::try_from(codepoint)
                    .ok()
                    .and_then(|index| glyphs.get(index))
                    .copied()
                    .map_or(0, Glyph::from)
            }
            Mapping::Segments(segments) => {
                let Ok(codepoint) = u16::try_from(codepoint) else {
                    return 0;
                };
                let Some(segment) = segments
                    .iter()
                    .rev()
                    .find(|segment| segment.start <= codepoint && codepoint <= segment.end)
                else {
                    return 0;
                };
                let glyph = match &segment.glyphs {
                    Some(glyphs) => {
                        let glyph = glyphs[usize::from(codepoint - segment.start)];
                        if glyph == 0 {
                            return 0;
                        }
                        i32::from(glyph)
                    }
                    None => i32::from(codepoint),
                };
                Glyph::from((glyph + i32::from(segment.delta)) & 65535)
            }
            Mapping::Groups {
                groups,
                incremental,
            } => {
                let Some(group) = groups
                    .iter()
                    .rev()
                    .find(|group| group.start <= codepoint && codepoint <= group.end)
                else {
                    return 0;
                };
                Glyph::from(group.glyph)
                    + if *incremental {
                        Glyph::from(codepoint - group.start)
                    } else {
                        0
                    }
            }
        }
    }
}

fn parse_segments(table: Reader<'_>) -> Result<Vec<Segment>, FontError> {
    let count = usize::from(table.u16(6)? >> 1);
    // Counts are u16 protocol fields, so these products fit every target usize.
    // Read only the segments used by the source, which skips the last one.
    if count > 1 {
        table.bytes(0, 14 + count * 8)?;
    }
    let mut segments = Vec::new();
    segments.try_reserve_exact(count.saturating_sub(1))?;
    for index in 0..count.saturating_sub(1) {
        let end = table.u16(14 + index * 2)?;
        let start = table.u16(16 + count * 2 + index * 2)?;
        let delta = table.i16(16 + count * 4 + index * 2)?;
        let offset_position = 16 + count * 6 + index * 2;
        let range_offset = usize::from(table.u16(offset_position)?);
        let glyphs = if range_offset != 0 && start <= end {
            Some(table.words(offset_position + range_offset, usize::from(end - start) + 1)?)
        } else {
            None
        };
        segments.push(Segment {
            start,
            end,
            delta,
            glyphs,
        });
    }
    Ok(segments)
}

fn validate_variations(table: Reader<'_>) -> Result<(), FontError> {
    let count = table.offset32(6)?;
    table.bytes(10, count.checked_mul(11).ok_or(FontError::Bounds)?)?;
    for index in 0..count {
        let offset = 10 + index * 11;
        for (relative, width) in [(3, 4), (7, 5)] {
            let target = table.offset32(offset + relative)?;
            if target != 0 {
                let mapping = table.at(target)?;
                let records = mapping.offset32(0)?;
                mapping.bytes(4, records.checked_mul(width).ok_or(FontError::Bounds)?)?;
            }
        }
    }
    Ok(())
}

fn parse_groups(table: Reader<'_>) -> Result<Vec<Group>, FontError> {
    let count = table.offset32(12)?;
    table.bytes(16, count.checked_mul(12).ok_or(FontError::Bounds)?)?;
    let mut groups = Vec::new();
    groups.try_reserve_exact(count)?;
    for index in 0..count {
        let offset = 16 + index * 12;
        groups.push(Group {
            start: table.u32(offset)?,
            end: table.u32(offset + 4)?,
            glyph: table.u32(offset + 8)?,
        });
    }
    Ok(groups)
}
