// Copyright (c) 2023, 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Native port of xterm-wasm-parts Base64Decoder.

//! Strict, deferred, in-place payload decoding shared by Kitty and IIP.
//! Clipboard/name decoding uses a different permissive policy. Non-byte input
//! truncates to its low eight bits, matching Uint8Array.set, before validation.

use std::collections::TryReserveError;
use std::error::Error;
use std::fmt;

const DECODE_THRESHOLD: usize = 131_072;
const MAX_BYTES: usize = 4_294_901_760;
const DATA_OFFSET: usize = 5_152;
const PAGE_BYTES: usize = 65_536;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeStatus {
    Ok,
    DecodeError,
    NotInitialized,
    SizeExceeded,
}
impl DecodeStatus {
    pub fn code(self) -> i32 {
        match self {
            Self::Ok => 0,
            Self::DecodeError => -1,
            Self::NotInitialized => -2,
            Self::SizeExceeded => -3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidSettings;
impl fmt::Display for InvalidSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid byte settings")
    }
}
impl Error for InvalidSettings {}

pub struct PayloadDecoder {
    pub keep_size: usize,
    pub max_bytes: usize,
    bytes: usize,
    physical_bytes: usize,
    data: Vec<u8>,
    initialized: bool,
    ended: bool,
    write: usize,
    source: usize,
    decoded: usize,
}
impl PayloadDecoder {
    pub fn new(
        keep_size: usize,
        max_bytes: usize,
        initial_bytes: usize,
    ) -> Result<Self, InvalidSettings> {
        if initial_bytes > max_bytes || max_bytes > MAX_BYTES {
            return Err(InvalidSettings);
        }
        Ok(Self {
            keep_size,
            max_bytes,
            bytes: initial_bytes,
            physical_bytes: 0,
            data: Vec::new(),
            initialized: false,
            ended: true,
            write: 0,
            source: 0,
            decoded: 0,
        })
    }

    pub fn init(
        &mut self,
        max_bytes: Option<usize>,
        initial_bytes: Option<usize>,
    ) -> Result<(), InvalidSettings> {
        self.max_bytes = max_bytes.unwrap_or(self.max_bytes);
        self.bytes = initial_bytes.unwrap_or(self.bytes.min(self.max_bytes));
        if self.bytes > self.max_bytes || self.max_bytes > MAX_BYTES {
            return Err(InvalidSettings);
        }
        self.physical_bytes = self.physical_bytes.max(physical_bytes(self.bytes));
        self.write = 0;
        self.source = 0;
        self.decoded = 0;
        self.initialized = true;
        self.ended = false;
        Ok(())
    }

    /// Preserve the source's keep-size policy without allocating every untouched
    /// WASM page eagerly. Logical limits, offsets and visible bytes remain exact.
    pub fn release(&mut self) {
        if !self.initialized {
            return;
        }
        if self.bytes > self.keep_size {
            self.data = Vec::new();
            self.initialized = false;
            self.physical_bytes = 0;
        } else {
            self.write = 0;
            self.source = 0;
            self.decoded = 0;
        }
    }

    pub fn data(&self) -> &[u8] {
        if self.initialized {
            &self.data[..self.decoded]
        } else {
            &[]
        }
    }
    pub fn loaded_bytes(&self) -> usize {
        if self.initialized {
            self.write
        } else {
            0
        }
    }
    pub fn free_bytes(&self) -> i128 {
        if self.initialized {
            self.max_bytes as i128 - self.write as i128
        } else {
            0
        }
    }

    fn ensure(&mut self, length: usize) -> Result<(), TryReserveError> {
        if length > self.data.len() {
            self.data.try_reserve(length - self.data.len())?;
            self.data.resize(length, 0);
        }
        Ok(())
    }

    pub fn put(&mut self, input: &[u32]) -> Result<DecodeStatus, TryReserveError> {
        if !self.initialized || self.ended {
            return Ok(DecodeStatus::NotInitialized);
        }
        // A caller may retain the object after a failed init or mutate the
        // public limit, as with upstream. Never let invalid settings reach
        // address arithmetic or an allocation attempt.
        if self.max_bytes > MAX_BYTES || self.bytes > self.max_bytes {
            return Ok(DecodeStatus::SizeExceeded);
        }
        let Some(needed) = self.write.checked_add(input.len()) else {
            return Ok(DecodeStatus::SizeExceeded);
        };
        if needed > self.bytes {
            if needed > self.max_bytes {
                return Ok(DecodeStatus::SizeExceeded);
            }
            // A zero initial size makes the upstream doubling loop non-terminating.
            // Grow from one here; image handlers reject nonempty zero-limit input
            // before decoder creation, so that source bug is not a protocol path.
            let mut grown = self.bytes.max(1);
            while grown < needed {
                grown = grown.saturating_mul(2);
            }
            self.bytes = grown.min(self.max_bytes);
            self.physical_bytes = self.physical_bytes.max(physical_bytes(self.bytes));
        }
        self.ensure(needed)?;
        for (target, value) in self.data[self.write..needed].iter_mut().zip(input) {
            *target = *value as u8;
        }
        self.write = needed;
        if self.write - self.source >= DECODE_THRESHOLD {
            self.decode_blocks()
        } else {
            Ok(DecodeStatus::Ok)
        }
    }

    fn packed(&self, position: usize, length: usize) -> u32 {
        (0..length).fold(0, |word, phase| {
            word | lookup(self.data[position + phase], phase)
        })
    }
    fn store_word(&mut self, position: usize, word: u32) -> Result<bool, TryReserveError> {
        let Some(end) = position
            .checked_add(4)
            .filter(|end| *end <= self.physical_bytes)
        else {
            return Ok(false);
        };
        self.ensure(end)?;
        self.data[position..end].copy_from_slice(&word.to_le_bytes());
        Ok(true)
    }
    fn decode_blocks(&mut self) -> Result<DecodeStatus, TryReserveError> {
        let end = (self.write - 1) & !3;
        let mut source = self.source;
        let mut decoded = self.decoded;
        let mut error = 0;
        while source < end {
            let word = self.packed(source, 4);
            if !self.store_word(decoded, word)? {
                return Ok(DecodeStatus::DecodeError);
            }
            error |= word;
            source += 4;
            decoded += 3;
        }
        if error >> 24 != 0 {
            return Ok(DecodeStatus::DecodeError);
        }
        self.source = end;
        self.decoded = decoded;
        Ok(DecodeStatus::Ok)
    }

    pub fn end(&mut self) -> Result<DecodeStatus, TryReserveError> {
        self.ended = true;
        if !self.initialized {
            return Ok(DecodeStatus::NotInitialized);
        }
        if self.write - self.source > 4 {
            let status = self.decode_blocks()?;
            if status != DecodeStatus::Ok {
                return Ok(status);
            }
        }
        let mut remainder = self.write - self.source;
        if remainder < 2 {
            return Ok(DecodeStatus::DecodeError);
        }
        if remainder == 4 {
            if self.data[self.source + 3] == b'=' {
                remainder -= 1;
            }
            if self.data[self.source + 2] == b'=' {
                remainder -= 1;
            }
        }
        let word = self.packed(self.source, remainder);
        if word >> 24 != 0 || !self.store_word(self.decoded, word)? {
            return Ok(DecodeStatus::DecodeError);
        }
        self.decoded += remainder - 1;
        Ok(DecodeStatus::Ok)
    }
}

#[cfg(test)]
#[path = "base64_tests.rs"]
mod tests;

fn physical_bytes(bytes: usize) -> usize {
    let total = (bytes as u64 + DATA_OFFSET as u64).div_ceil(PAGE_BYTES as u64) * PAGE_BYTES as u64
        - DATA_OFFSET as u64;
    // The validated source maximum leaves room for the prefix below 2^32.
    // Calculate in u64 so a full 4-GiB page boundary also works on wasm32.
    usize::try_from(total).expect("validated decoder data capacity fits 32-bit address space")
}

fn lookup(byte: u8, phase: usize) -> u32 {
    let digit = match byte {
        b'A'..=b'Z' => byte - b'A',
        b'a'..=b'z' => byte - b'a' + 26,
        b'0'..=b'9' => byte - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => return 0xff00_0000,
    };
    let digit = u32::from(digit);
    match phase {
        0 => digit << 2,
        1 => (digit >> 4) | (((digit << 4) & 255) << 8),
        2 => ((digit >> 2) << 8) | (((digit << 6) & 255) << 16),
        _ => digit << 16,
    }
}
