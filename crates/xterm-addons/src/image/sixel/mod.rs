// Copyright (c) 2021 Joerg Breitbart. MIT license.
// Rust port of node-sixel 0.16.0 Decoder.js and wasm/decoder.cpp.

mod background;
mod canvas;
mod colors;
mod handler;
#[cfg(test)]
mod oracle_tests;
mod parser;
mod storage;
#[cfg(test)]
mod tests;
mod types;
pub use background::{active_background, ActiveColors, BackgroundError};
pub use colors::ADDON_PALETTE;
pub use handler::{SixelEnd, SixelEndPlan, SixelHandler, SixelPut, SixelWarning};
pub use storage::{add_sixel_image, advance_sixel_cursor};
pub use types::{
    DecodeError, DecoderOptions, DecoderProperties, RasterAttributes, SixelLevel, SixelMode,
};

use super::types::RgbaImage;
use std::cell::Cell;

pub const MAX_WIDTH: usize = 16_384;
pub const MAX_VISIBLE_WIDTH: usize = MAX_WIDTH - 4;
pub const PALETTE_SIZE: usize = 4096;
pub const CHUNK_SIZE: usize = 16_384;
const STRIDE: usize = MAX_WIDTH + 4;
// The original reports seven fixed WASM pages, not just live pixels. Retain
// that observable accounting for the handler's release threshold, not as a
// claim about the native allocator's physical memory usage.
const SOURCE_STATIC_BYTES: usize = 7 * 65_536;
// The pinned WASM places p0's four reserved words at byte33952. Preserve the
// remaining sandbox tail for checked emulation of malformed memcpy/repeats.
const SOURCE_PIXEL_OFFSET_BYTES: usize = 33_952;
const SOURCE_PIXEL_MEMORY_WORDS: usize = (SOURCE_STATIC_BYTES - SOURCE_PIXEL_OFFSET_BYTES) / 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Data,
    Repeat,
    Raster,
    Color,
}

/// Safe streaming implementation of the bundled SIXEL decoder.
/// Pixel words are explicitly ABGR values; RGBA output uses little-endian bytes
/// independently of the host CPU's byte order.
pub struct SixelDecoder {
    options: DecoderOptions,
    palette: Vec<u32>,
    band: Vec<u32>,
    canvas: Vec<u32>,
    band_widths: Vec<usize>,
    max_width: usize,
    min_width: usize,
    last_offset: usize,
    current_height: i64,
    raster: RasterAttributes,
    mode: SixelMode,
    level: SixelLevel,
    state: State,
    truncate: bool,
    fill: u32,
    color: u32,
    palette_limit: u32,
    params: [i32; 8],
    param_len: usize,
    cursor: i32,
    real_width: Cell<i32>,
    cleared_width: usize,
    band_bits: u8,
}

impl SixelDecoder {
    pub fn new(options: DecoderOptions) -> Result<Self, DecodeError> {
        if options.palette_limit > PALETTE_SIZE as f64 {
            return Err(DecodeError::PaletteTooLarge);
        }
        let mut palette = canvas::allocate(PALETTE_SIZE, 0)?;
        let length = options.palette.len().min(PALETTE_SIZE);
        palette[..length].copy_from_slice(&options.palette[..length]);
        let band = canvas::allocate(SOURCE_PIXEL_MEMORY_WORDS, 0)?;
        let color = options.foreground;
        let palette_limit = types::js_uint32(options.palette_limit).min(PALETTE_SIZE as u32);
        Ok(Self {
            options,
            palette,
            band,
            canvas: Vec::new(),
            band_widths: Vec::new(),
            max_width: 0,
            min_width: MAX_WIDTH,
            last_offset: 0,
            current_height: 0,
            raster: RasterAttributes::default(),
            mode: SixelMode::Undecided,
            level: SixelLevel::Undecided,
            state: State::Data,
            truncate: false,
            fill: 0,
            color,
            palette_limit,
            params: [0; 8],
            param_len: 1,
            cursor: 4,
            real_width: Cell::new(4),
            cleared_width: 4,
            band_bits: 0,
        })
    }

    /// `None` retains palette mutations, matching the image handler's init call.
    pub fn init(&mut self, fill: u32, palette: Option<&[u32]>, palette_limit: f64, truncate: bool) {
        if let Some(palette) = palette {
            let length = palette.len().min(PALETTE_SIZE);
            self.palette[..length].copy_from_slice(&palette[..length]);
        }
        self.init_state(self.options.foreground, fill, palette_limit, truncate);
        self.band_widths.clear();
        self.max_width = 0;
        self.min_width = MAX_WIDTH;
        self.last_offset = 0;
        self.current_height = 0;
    }

    pub fn init_default(&mut self) {
        let length = self.options.palette.len().min(PALETTE_SIZE);
        self.palette[..length].copy_from_slice(&self.options.palette[..length]);
        self.init(
            self.options.fill,
            None,
            self.options.palette_limit,
            self.options.truncate,
        );
    }

    fn init_state(&mut self, foreground: u32, fill: u32, palette_limit: f64, truncate: bool) {
        self.state = State::Data;
        self.color = foreground;
        self.cursor = 4;
        self.palette_limit = types::js_uint32(palette_limit).min(PALETTE_SIZE as u32);
        self.params[0] = 0;
        self.param_len = 1;
        self.truncate = truncate;
        self.level = SixelLevel::Undecided;
        self.mode = SixelMode::Undecided;
        self.fill = fill;
        self.raster = RasterAttributes::default();
        self.band_bits = 0;
    }

    pub fn release(&mut self) {
        self.canvas = Vec::new();
        self.band_widths.clear();
        self.max_width = 0;
        self.min_width = MAX_WIDTH;
        self.init_state(u32::MAX, 0, self.options.palette_limit, false);
    }

    pub fn clear_palette(&mut self) {
        self.palette.fill(0);
    }
    pub fn palette(&self) -> &[u32] {
        &self.palette[..self.palette_limit as usize]
    }
    pub fn mode(&self) -> SixelMode {
        self.mode
    }

    pub fn width(&self) -> i64 {
        match self.mode {
            SixelMode::Undecided => 0,
            SixelMode::Truncated => {
                let stored = ((self.raster.width as i32).min(MAX_WIDTH as i32) + 4) as u32;
                if stored == 0 {
                    0
                } else {
                    i64::from(stored) - 4
                }
            }
            SixelMode::Dynamic => i64::from(self.current_width()).max(self.max_width as i64),
        }
    }

    pub fn height(&self) -> i64 {
        match self.mode {
            SixelMode::Undecided => 0,
            SixelMode::Truncated => i64::from(self.raster.height),
            // Vec<usize> length is bounded by isize::MAX / size_of::<usize>(),
            // so six rows per stored band fit i64 on supported 32/64-bit hosts.
            SixelMode::Dynamic => {
                self.band_widths.len() as i64 * 6
                    + if self.current_width() != 0 {
                        i64::from(self.band_height())
                    } else {
                        0
                    }
            }
        }
    }

    fn current_width(&self) -> i32 {
        let width = self.cursor.max(self.real_width.get()).min(MAX_WIDTH as i32);
        self.real_width.set(width);
        width - 4
    }

    fn band_height(&self) -> i32 {
        (u8::BITS - self.band_bits.leading_zeros()) as i32
    }

    pub fn memory_usage(&self) -> Result<usize, DecodeError> {
        self.canvas
            .len()
            .checked_mul(4)
            .and_then(|bytes| {
                self.band_widths
                    .len()
                    .checked_mul(8)
                    .and_then(|bands| bytes.checked_add(bands))
            })
            .and_then(|bytes| bytes.checked_add(SOURCE_STATIC_BYTES))
            .ok_or(DecodeError::Arithmetic)
    }

    pub fn properties(&self) -> Result<DecoderProperties, DecodeError> {
        Ok(DecoderProperties {
            width: self.width(),
            height: self.height(),
            mode: self.mode,
            level: self.level,
            truncate: self.truncate,
            palette_limit: self.palette_limit,
            fill_color: self.fill,
            memory_usage: self.memory_usage()?,
            raster: self.raster,
        })
    }

    /// Typed-array input is truncated to one byte and then to seven bits, as the
    /// reference's Uint8Array copy and C decoder do. Chunk size is source data.
    pub fn decode<T: Copy + Into<u32>>(&mut self, data: &[T]) -> Result<(), DecodeError> {
        for chunk in data.chunks(CHUNK_SIZE) {
            self.decode_chunk(chunk)?;
        }
        Ok(())
    }

    pub fn image(&mut self) -> Result<RgbaImage, DecodeError> {
        // data8 evaluates data32 before reading the final byte-view dimensions.
        let pixels = self.pixels()?;
        let width = u32::try_from(self.width()).map_err(|_| DecodeError::InvalidDimensions)?;
        let height = u32::try_from(self.height()).map_err(|_| DecodeError::InvalidDimensions)?;
        let length = pixels.len().checked_mul(4).ok_or(DecodeError::Arithmetic)?;
        let mut rgba = Vec::new();
        rgba.try_reserve_exact(length)
            .map_err(|_| DecodeError::Allocation)?;
        for pixel in pixels {
            rgba.extend_from_slice(&pixel.to_le_bytes());
        }
        RgbaImage::new(width, height, rgba).map_err(DecodeError::Raster)
    }
}
