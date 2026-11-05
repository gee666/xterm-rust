// Constructor-only metadata checks follow opentype.js. See licenses/opentype-js-MIT.txt.
// These tables do not affect shaping, but malformed data must still reject loading.
use super::font_bytes::{FontError, Reader};
#[path = "font_name_languages.rs"]
mod languages;
const HEAD_MAGIC: u32 = 0x5f0f_3cf5;
const STANDARD_POST_NAMES: u16 = 258;
const META_VERSION: u32 = 1;
const META_HEADER_SIZE: usize = 16;
const META_MAP_SIZE: usize = 12;

pub fn immediate(tag: &[u8], bytes: &[u8]) -> Result<(), FontError> {
    let table = Reader::new(bytes);
    match tag {
        b"head" => {
            table.bytes(0, 16)?;
            if table.u32(12)? != HEAD_MAGIC {
                return Err(FontError::Invalid("Font header has wrong magic number."));
            }
            table.bytes(16, 38)?;
        }
        b"hhea" => {
            table.bytes(0, 36)?;
        }
        b"maxp" => {
            table.bytes(0, 6)?;
            // parseVersion uses decimal minor units, not a fixed-point number.
            if version(table)? == 1.0 {
                table.bytes(6, 26)?;
            }
        }
        b"OS/2" => {
            let version = table.u16(0)?;
            let size = if version >= 2 {
                96
            } else if version == 1 {
                86
            } else {
                78
            };
            table.bytes(0, size)?;
        }
        b"post" => post(table, 0)?,
        _ => return Err(FontError::Invalid("Unknown fixed metadata table")),
    }
    Ok(())
}

fn version(table: Reader<'_>) -> Result<f64, FontError> {
    Ok(f64::from(table.u16(0)?) + f64::from(table.u16(2)?) / 40960.0)
}

pub fn post(source: Reader<'_>, start: usize) -> Result<(), FontError> {
    let table = source.at(start)?;
    let version = version(table)?;
    table.bytes(0, 32)?;
    if version == 2.0 {
        let count = usize::from(table.u16(32)?);
        let indices = table.words(34, count)?;
        let mut cursor = i64::try_from(start + 34 + count * 2).map_err(|_| FontError::Bounds)?;
        // The source reads one Pascal string for *each* custom index, including
        // repeated indices. Its signed parseChar also moves backwards for a
        // negative length, so deliberately preserve that cursor arithmetic.
        for index in indices {
            if index >= STANDARD_POST_NAMES {
                let at = usize::try_from(cursor).map_err(|_| FontError::Bounds)?;
                let length = i8::from_be_bytes([source.bytes(at, 1)?[0]]);
                cursor += 1;
                if length > 0 {
                    source.bytes(at + 1, length as usize)?;
                }
                cursor += i64::from(length);
            }
        }
    } else if version == 2.5 {
        let count = usize::from(table.u16(32)?);
        table.bytes(34, count)?;
        // opentype.js reads glyphNameIndex here, although the 2.5 parser only
        // creates offset. Preserve its constructor failure instead of fixing it.
        if count > 0 {
            return Err(FontError::Type(
                "Cannot read properties of undefined (reading '0')",
            ));
        }
    }
    Ok(())
}

pub fn ltag(table: Reader<'_>) -> Result<usize, FontError> {
    if table.u32(0)? != 1 {
        return Err(FontError::Invalid("Unsupported ltag table version."));
    }
    let count = table.offset32(8)?;
    table.bytes(12, count.checked_mul(4).ok_or(FontError::Bounds)?)?;
    for index in 0..count {
        let offset = usize::from(table.u16(12 + index * 4)?);
        let length = usize::from(table.u16(14 + index * 4)?);
        // A zero-length string does not access its offset in JavaScript.
        if length > 0 {
            table.bytes(offset, length)?;
        }
    }
    Ok(count)
}

pub fn meta(bytes: &[u8]) -> Result<(), FontError> {
    let table = Reader::new(bytes);
    if table.u32(0)? != META_VERSION {
        return Err(FontError::Invalid("Unsupported META table version."));
    }
    // Flags and the string-pool offset are read but not interpreted upstream.
    table.u32(4)?;
    table.u32(8)?;
    let count = table.offset32(12)?;
    for index in 0..count {
        let record = META_HEADER_SIZE
            .checked_add(index.checked_mul(META_MAP_SIZE).ok_or(FontError::Bounds)?)
            .ok_or(FontError::Bounds)?;
        let tag = table.bytes(record, 4)?;
        let offset = table.offset32(record + 4)?;
        let length = table.offset32(record + 8)?;
        // Source META UTF8 decoding reads individual bytes without validating
        // UTF8. Ligatures do not consume its strings, so retain only reads and
        // errors, including skipped binary tags and zero-read offsets.
        if !matches!(tag, b"appl" | b"bild") && length != 0 {
            table.bytes(offset, length)?;
        }
    }
    Ok(())
}

pub fn name(bytes: &[u8], tags: Option<usize>) -> Result<(), FontError> {
    let table = Reader::new(bytes);
    let format = table.u16(0)?;
    let count = usize::from(table.u16(2)?);
    let strings = usize::from(table.u16(4)?);
    table.bytes(6, count * 12)?;
    for index in 0..count {
        let record = table.at(6 + index * 12)?;
        let platform = record.u16(0)?;
        let encoding = record.u16(2)?;
        let language = record.u16(4)?;
        let length = usize::from(record.u16(8)?);
        let offset = strings + usize::from(record.u16(10)?);
        let unicode = match platform {
            0 => language == u16::MAX || tags.is_some_and(|tags| usize::from(language) < tags),
            3 => [1, 10].contains(&encoding) && languages::windows(language),
            _ => false,
        };
        let mac = platform == 1
            && languages::mac(language)
            && super::font_cmap::mac::mac_encoding(platform, encoding, i32::from(language))
                .is_some();
        if length > 0 {
            if unicode {
                // UTF16 loops once per two bytes, including an odd trailing byte.
                table.bytes(offset, length + length % 2)?;
            } else if mac {
                table.bytes(offset, length)?;
            }
        }
    }
    if format == 1 {
        table.u16(6 + count * 12)?;
    }
    Ok(())
}
