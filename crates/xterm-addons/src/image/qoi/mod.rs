// Copyright (c) 2023 The xterm.js authors. MIT license.
//! Safe native port of xterm-wasm-parts QoiDecoder, including reused memory.
mod memory;
mod parser;
use super::types::{RasterError, RgbaImage};
use std::collections::TryReserveError;
use std::fmt;

const PAGE_BYTES: usize = 65_536;
const OUTPUT_OFFSET: usize = 1024;
const MEMORY_PADDING: f64 = 4096.0;
const MAX_PAGES: f64 = 65_536.0;

#[derive(Debug)]
pub enum QoiError {
    Allocation(TryReserveError),
    MemorySize,
    MemoryInitial,
    MemoryImport,
    MemoryAccess,
    InputOffset,
    InvalidDimensions,
    Raster(RasterError),
}
impl fmt::Display for QoiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allocation(e) => e.fmt(f),
            Self::MemorySize => f.write_str("invalid QOI memory size"),
            Self::MemoryInitial => f.write_str("invalid QOI initial memory pages"),
            Self::MemoryImport => f.write_str("QOI decoder memory import requires one page"),
            Self::MemoryAccess => f.write_str("memory access out of bounds"),
            Self::InputOffset => f.write_str("offset is out of bounds"),
            Self::InvalidDimensions => f.write_str("invalid QOI image dimensions"),
            Self::Raster(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for QoiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Allocation(e) => Some(e),
            Self::Raster(e) => Some(e),
            _ => None,
        }
    }
}
impl QoiError {
    pub fn js_name(&self) -> &'static str {
        match self {
            Self::MemoryAccess => "RuntimeError",
            Self::MemoryInitial => "TypeError",
            Self::MemoryImport => "LinkError",
            _ => "RangeError",
        }
    }
}
impl From<TryReserveError> for QoiError {
    fn from(value: TryReserveError) -> Self {
        Self::Allocation(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct QoiImage {
    pub width: i32,
    pub height: i32,
    pub pixels: Vec<u8>,
}
impl QoiImage {
    pub fn into_rgba(self) -> Result<RgbaImage, QoiError> {
        if self.width <= 0 || self.height <= 0 {
            return Err(QoiError::InvalidDimensions);
        }
        let width = u32::try_from(self.width).map_err(|_| QoiError::InvalidDimensions)?;
        let height = u32::try_from(self.height).map_err(|_| QoiError::InvalidDimensions)?;
        RgbaImage::new(width, height, self.pixels).map_err(QoiError::Raster)
    }
}

pub struct QoiDecoder {
    pub keep_size: f64,
    pub width: i32,
    pub height: i32,
    memory: Vec<u8>,
    initialized: bool,
}
impl QoiDecoder {
    pub fn new(keep_size: f64) -> Self {
        Self {
            keep_size,
            width: 0,
            height: 0,
            memory: Vec::new(),
            initialized: false,
        }
    }
    pub fn memory_usage(&self) -> usize {
        self.memory.len()
    }
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    pub fn release(&mut self) {
        if self.initialized && self.memory.len() as f64 > self.keep_size {
            self.memory = Vec::new();
            self.initialized = false;
        }
    }
    pub fn decode(&mut self, input: &[u8]) -> Result<QoiImage, QoiError> {
        self.width = dimension(input, 4);
        self.height = dimension(input, 8);
        let pixels = f64::from(self.width) * f64::from(self.height);
        let output_bytes = pixels * 4.0;
        let length = input.len() as f64;
        // The signed shift is part of the actual wrapper's size expression.
        let needed = output_bytes.max(length)
            + f64::from(js_i32(output_bytes.min(length)) >> 1)
            + MEMORY_PADDING;
        self.ensure_memory(needed)?;
        let difference = self
            .memory
            .len()
            .checked_sub(input.len())
            .ok_or(QoiError::InputOffset)?;
        let offset = js_i32(difference as f64) & !255;
        let offset = usize::try_from(offset).map_err(|_| QoiError::InputOffset)?;
        let end = offset
            .checked_add(input.len())
            .ok_or(QoiError::InputOffset)?;
        self.memory
            .get_mut(offset..end)
            .ok_or(QoiError::InputOffset)?
            .copy_from_slice(input);
        self.decode_memory(offset, input.len(), js_i32(pixels))?;
        let bound = OUTPUT_OFFSET as f64 + output_bytes;
        let end = if bound < 0.0 {
            (self.memory.len() as f64 + bound).max(0.0)
        } else {
            bound
        }
        .min(self.memory.len() as f64) as usize;
        let start = OUTPUT_OFFSET.min(self.memory.len());
        let data = if end > start {
            &self.memory[start..end]
        } else {
            &[]
        };
        let mut result = Vec::new();
        result.try_reserve_exact(data.len())?;
        result.extend_from_slice(data);
        Ok(QoiImage {
            width: self.width,
            height: self.height,
            pixels: result,
        })
    }
}
fn dimension(data: &[u8], offset: usize) -> i32 {
    i32::from_be_bytes(std::array::from_fn(|index| {
        data.get(offset + index).copied().unwrap_or(0)
    }))
}
fn js_i32(value: f64) -> i32 {
    if !value.is_finite() || value == 0.0 {
        return 0;
    }
    let unsigned = value.trunc().rem_euclid(4_294_967_296.0) as u32;
    i32::from_ne_bytes(unsigned.to_ne_bytes())
}
