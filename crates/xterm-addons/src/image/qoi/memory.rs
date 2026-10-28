// Copyright (c) 2023 The xterm.js authors. MIT license.
use super::{QoiDecoder, QoiError, MAX_PAGES, PAGE_BYTES};
impl QoiDecoder {
    pub(super) fn ensure_memory(&mut self, needed: f64) -> Result<(), QoiError> {
        if !self.initialized {
            let pages = (needed / PAGE_BYTES as f64).ceil();
            if pages < 0.0 {
                return Err(QoiError::MemoryInitial);
            }
            let length = memory_length(pages)?;
            let mut memory = Vec::new();
            memory.try_reserve_exact(length)?;
            memory.resize(length, 0);
            self.memory = memory;
            // The shipped WASM import declares a minimum of one page; an
            // allocated zero-page Memory fails linking before the start function.
            if length < PAGE_BYTES {
                return Err(QoiError::MemoryImport);
            }
            self.initialized = true;
        } else if (self.memory.len() as f64) < needed {
            let extra = ((needed - self.memory.len() as f64) / PAGE_BYTES as f64).ceil();
            let length = self
                .memory
                .len()
                .checked_add(memory_length(extra)?)
                .ok_or(QoiError::MemorySize)?;
            if length as f64 > MAX_PAGES * PAGE_BYTES as f64 {
                return Err(QoiError::MemorySize);
            }
            self.memory.try_reserve_exact(length - self.memory.len())?;
            self.memory.resize(length, 0);
        }
        Ok(())
    }
    pub(super) fn byte(&self, position: u32) -> Result<u8, QoiError> {
        self.memory
            .get(position as usize)
            .copied()
            .ok_or(QoiError::MemoryAccess)
    }
    pub(super) fn pixel(&self, position: u32) -> Result<[u8; 4], QoiError> {
        let start = position as usize;
        let end = start.checked_add(4).ok_or(QoiError::MemoryAccess)?;
        let data = self.memory.get(start..end).ok_or(QoiError::MemoryAccess)?;
        Ok([data[0], data[1], data[2], data[3]])
    }
    pub(super) fn set_pixel(&mut self, position: u32, pixel: [u8; 4]) -> Result<(), QoiError> {
        let start = position as usize;
        let end = start.checked_add(4).ok_or(QoiError::MemoryAccess)?;
        self.memory
            .get_mut(start..end)
            .ok_or(QoiError::MemoryAccess)?
            .copy_from_slice(&pixel);
        Ok(())
    }
}
fn memory_length(pages: f64) -> Result<usize, QoiError> {
    // WebAssembly converts the page argument to unsigned long before checking
    // its separate 65,536-page memory limit, including Memory.grow arguments.
    if pages > f64::from(u32::MAX) {
        return Err(QoiError::MemoryArgument);
    }
    if !pages.is_finite() || !(0.0..=MAX_PAGES).contains(&pages) {
        return Err(QoiError::MemorySize);
    }
    let bytes = (pages as u64)
        .checked_mul(PAGE_BYTES as u64)
        .ok_or(QoiError::MemorySize)?;
    usize::try_from(bytes)
        .ok()
        .filter(|n| *n <= isize::MAX as usize)
        .ok_or(QoiError::MemorySize)
}
