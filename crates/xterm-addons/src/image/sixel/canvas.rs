// Copyright (c) 2021 Joerg Breitbart. MIT license.
// Band assembly follows the original node-sixel Decoder.js allocation policy.
use super::{DecodeError, SixelDecoder, SixelLevel, SixelMode, MAX_WIDTH, STRIDE};

pub(super) fn allocate(length: usize, fill: u32) -> Result<Vec<u32>, DecodeError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(length)
        .map_err(|_| DecodeError::Allocation)?;
    result.resize(length, fill);
    Ok(result)
}

fn copy(data: &[u32]) -> Result<Vec<u32>, DecodeError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(data.len())
        .map_err(|_| DecodeError::Allocation)?;
    result.extend_from_slice(data);
    Ok(result)
}

impl SixelDecoder {
    fn check_memory(&mut self, pixels: f64) -> Result<(), DecodeError> {
        let bytes = pixels * 4.0;
        if self
            .options
            .memory_limit
            .is_some_and(|limit| limit != 0.0 && bytes > limit)
        {
            self.release();
            return Err(DecodeError::MemoryLimit);
        }
        Ok(())
    }

    pub(super) fn prepare_canvas(&mut self) -> Result<(), DecodeError> {
        let hint = if self.mode == SixelMode::Truncated {
            self.width() as f64 * self.height() as f64
        } else if self.level == SixelLevel::Two {
            f64::from(self.raster.width.min(MAX_WIDTH as u32)) * f64::from(self.raster.height)
        } else {
            if self.canvas.len() < 65_536 {
                self.canvas = allocate(65_536, 0)?;
            }
            return Ok(());
        };
        if hint > self.canvas.len() as f64 {
            self.check_memory(hint)?;
            // Vec cannot address more than isize::MAX bytes. Check before the
            // float-to-integer boundary instead of accepting a saturating cast.
            if hint > (isize::MAX as usize / 4) as f64 {
                return Err(DecodeError::Allocation);
            }
            self.canvas = allocate(hint as usize, 0)?;
        }
        Ok(())
    }

    fn grow_canvas(&mut self, offset: usize, additional: usize) -> Result<(), DecodeError> {
        let pixels = offset
            .checked_add(additional)
            .ok_or(DecodeError::Arithmetic)?;
        if pixels > self.canvas.len() {
            self.check_memory(pixels as f64)?;
            let rounded = pixels.checked_add(65_535).ok_or(DecodeError::Arithmetic)? / 65_536;
            let rounded = rounded.checked_mul(65_536).ok_or(DecodeError::Arithmetic)?;
            let mut next = allocate(rounded, 0)?;
            next[..self.canvas.len()].copy_from_slice(&self.canvas);
            self.canvas = next;
        }
        Ok(())
    }

    fn copy_band_rows(&mut self, width: usize, rows: usize) -> Result<(), DecodeError> {
        for row in 0..rows {
            let source = row * STRIDE + 4;
            let target = self
                .last_offset
                .checked_add(width.checked_mul(row).ok_or(DecodeError::Arithmetic)?)
                .ok_or(DecodeError::Arithmetic)?;
            let end = target.checked_add(width).ok_or(DecodeError::Arithmetic)?;
            let data = self
                .band
                .get(source..source + width)
                .ok_or(DecodeError::InvalidDimensions)?;
            self.canvas
                .get_mut(target..end)
                .ok_or(DecodeError::InvalidDimensions)?
                .copy_from_slice(data);
        }
        Ok(())
    }

    pub(super) fn finish_band(&mut self) -> Result<(), DecodeError> {
        let width = if self.mode == SixelMode::Truncated {
            self.width()
        } else {
            i64::from(self.current_width())
        };
        if self.mode == SixelMode::Truncated {
            let rows = (self.height() - self.current_height).clamp(0, 6) as usize;
            if rows == 0 {
                return Ok(());
            }
            let width = usize::try_from(width).map_err(|_| DecodeError::InvalidDimensions)?;
            self.copy_band_rows(width, rows)?;
            self.last_offset += width * rows;
            self.current_height += rows as i64;
        } else {
            let width = usize::try_from(width).map_err(|_| DecodeError::InvalidDimensions)?;
            self.grow_canvas(self.last_offset, width * 6)?;
            self.max_width = self.max_width.max(width);
            self.min_width = self.min_width.min(width);
            self.copy_band_rows(width, 6)?;
            self.band_widths
                .try_reserve(1)
                .map_err(|_| DecodeError::Allocation)?;
            self.band_widths.push(width);
            self.last_offset += width * 6;
            self.current_height += 6;
        }
        Ok(())
    }

    /// Return an owned snapshot; the JS wrapper's fast and realignment paths
    /// still run so their retained-buffer growth and memory errors are preserved.
    pub fn pixels(&mut self) -> Result<Vec<u32>, DecodeError> {
        if self.mode == SixelMode::Undecided || self.width() == 0 || self.height() == 0 {
            return Ok(Vec::new());
        }
        let width = usize::try_from(self.width()).map_err(|_| DecodeError::InvalidDimensions)?;
        let height = usize::try_from(self.height()).map_err(|_| DecodeError::InvalidDimensions)?;
        let length = width.checked_mul(height).ok_or(DecodeError::Arithmetic)?;
        let current_width = if self.mode == SixelMode::Truncated {
            width
        } else {
            self.current_width() as usize
        };
        if self.mode == SixelMode::Truncated {
            let remaining = (self.height() - self.current_height).max(0) as usize;
            let rows = remaining.min(6);
            self.copy_band_rows(current_width, rows)?;
            if remaining > rows {
                let start = self.last_offset + current_width * rows;
                self.canvas
                    .get_mut(start..)
                    .ok_or(DecodeError::InvalidDimensions)?
                    .fill(self.fill);
            }
            return copy(
                self.canvas
                    .get(..length)
                    .ok_or(DecodeError::InvalidDimensions)?,
            );
        }
        if self.min_width == self.max_width
            && (current_width == 0 || current_width == self.min_width)
        {
            if current_width != 0 {
                self.grow_canvas(self.last_offset, current_width * 6)?;
                self.copy_band_rows(current_width, 6)?;
            }
            return copy(
                self.canvas
                    .get(..length)
                    .ok_or(DecodeError::InvalidDimensions)?,
            );
        }
        let mut result = allocate(length, self.fill)?;
        let mut source = 0;
        let mut target = 0;
        for &band_width in &self.band_widths {
            for _ in 0..6 {
                let end = source + band_width;
                result
                    .get_mut(target..target + band_width)
                    .ok_or(DecodeError::InvalidDimensions)?
                    .copy_from_slice(
                        self.canvas
                            .get(source..end)
                            .ok_or(DecodeError::InvalidDimensions)?,
                    );
                source = end;
                target += width;
            }
        }
        if current_width != 0 {
            for row in 0..self.band_height() as usize {
                let source = row * STRIDE + 4;
                let target = target + width * row;
                result
                    .get_mut(target..target + current_width)
                    .ok_or(DecodeError::InvalidDimensions)?
                    .copy_from_slice(
                        self.band
                            .get(source..source + current_width)
                            .ok_or(DecodeError::InvalidDimensions)?,
                    );
            }
        }
        Ok(result)
    }
}
