// Constructor reads follow opentype.js. See licenses/opentype-js-MIT.txt.
use super::{FontError, Reader};
use miniz_oxide::{
    inflate::stream::{inflate, InflateState},
    DataFormat, MZFlush, MZStatus,
};

// Hint values never affect ligature lookup, so validate constructor reads
// without allocating a discarded list proportional to an untrusted length.
pub(super) fn validate(
    reader: Reader<'_>,
    record: Reader<'_>,
    woff: bool,
    words: bool,
) -> Result<(), FontError> {
    let length = record.offset32(12)?;
    let compressed = woff && record.offset32(8)? < length;
    if compressed {
        let offset = record
            .offset32(4)?
            .checked_add(2)
            .ok_or(FontError::Bounds)?;
        let size = record
            .offset32(8)?
            .checked_sub(2)
            .ok_or(FontError::Bounds)?;
        // Source skips the zlib header and ignores its checksum and output
        // length. Its fixed typed array zero-fills short output and discards
        // overflow. Only decompressor failure can reject the discarded hints.
        validate_deflate(reader.bytes(offset, size)?)?;
    }
    // parseShortList allocates Array(length / 2) after decompression. Odd
    // directory lengths throw before any DataView reads, even at bad offsets.
    if words && length % 2 != 0 {
        return Err(FontError::ArrayLength);
    }
    if !compressed && length != 0 {
        reader.bytes(record.offset32(if woff { 4 } else { 8 })?, length)?;
    }
    // Zero-element uncompressed lists never dereference their table offset.
    Ok(())
}

fn validate_deflate(mut input: &[u8]) -> Result<(), FontError> {
    // Source reads past the typed array as undefined. Bitwise operations and
    // destination writes turn that into zero, including missing stored payload.
    // A stored block has at most u16::MAX bytes plus its four-byte header. This
    // finite tail completes a final stored block or reaches a later invalid
    // zero header, without following the source into infinite virtual input.
    const VIRTUAL_INPUT_LIMIT: usize = u16::MAX as usize + 4;
    let mut state = InflateState::new_boxed(DataFormat::Raw);
    let mut discard = [0; 4096];
    let zeroes = [0; 4096];
    let mut virtual_read = 0;
    loop {
        let virtual_input = input.is_empty();
        let source = if virtual_input {
            let remaining = VIRTUAL_INPUT_LIMIT - virtual_read;
            if remaining == 0 {
                return Err(FontError::Compression);
            }
            &zeroes[..remaining.min(zeroes.len())]
        } else {
            input
        };
        let result = inflate(&mut state, source, &mut discard, MZFlush::None);
        if virtual_input {
            virtual_read += result.bytes_consumed;
        } else {
            input = &input[result.bytes_consumed..];
        }
        match result.status {
            Ok(MZStatus::StreamEnd) => return Ok(()),
            Ok(_) if result.bytes_consumed != 0 || result.bytes_written != 0 => {}
            _ => return Err(FontError::Compression),
        }
    }
}
