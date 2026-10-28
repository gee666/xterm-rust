// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{ImageStorage, StorageError};

const PIXELS_PER_MEGABYTE: f64 = 250_000.0;
impl ImageStorage {
    pub fn get_limit(&self) -> f64 {
        f64::from(self.inner.pixel_limit.get()) / PIXELS_PER_MEGABYTE
    }
    pub fn set_limit(&self, megabytes: f64) -> Result<(), StorageError> {
        // The source comparison deliberately accepts NaN, then >>>0 yields zero.
        if !megabytes.is_nan() && !(0.5..=1000.0).contains(&megabytes) {
            return Err(StorageError::InvalidLimit);
        }
        let pixels = to_u32(megabytes / 4.0 * 1_000_000.0);
        self.inner.pixel_limit.set(pixels);
        self.evict_oldest(0.0)?;
        Ok(())
    }
    pub fn get_usage(&self) -> Result<f64, StorageError> {
        Ok(self.stored_pixels()? / PIXELS_PER_MEGABYTE)
    }
    fn stored_pixels(&self) -> Result<f64, StorageError> {
        let mut cursor = None;
        let mut pixels = 0.0;
        while let Some((_, spec)) = self.next_image(&mut cursor) {
            pixels += f64::from(spec.original.width()?) * f64::from(spec.original.height()?);
            let actual = spec.actual.borrow().clone();
            if let Some(actual) = actual.filter(|actual| !actual.same_identity(&spec.original)) {
                pixels += f64::from(actual.width()?) * f64::from(actual.height()?);
            }
        }
        Ok(pixels)
    }
    pub(super) fn evict_oldest(&self, room: f64) -> Result<f64, StorageError> {
        let used = self.stored_pixels()?;
        let mut current = used;
        while f64::from(self.inner.pixel_limit.get()) < current + room && self.image_count() != 0 {
            let Some((id, spec)) = self.next_eviction() else {
                return Err(StorageError::IdentifierOrder);
            };
            // Skipping absent IDs has no side effects; commit the same final
            // lowest ID the source reaches by incrementing through sparse gaps.
            self.inner.lowest_id.set(id);
            current -= f64::from(spec.original.width()?) * f64::from(spec.original.height()?);
            let actual = spec.actual.borrow().clone();
            if let Some(actual) = actual.filter(|actual| !actual.same_identity(&spec.original)) {
                current -= f64::from(actual.width()?) * f64::from(actual.height()?);
            }
            if let Some(marker) = &spec.marker {
                marker.dispose()?;
            }
            self.delete_entry(id)?;
        }
        Ok(used - current)
    }
}
fn to_u32(value: f64) -> u32 {
    if !value.is_finite() || value == 0.0 {
        return 0;
    }
    value.trunc().rem_euclid(4_294_967_296.0) as u32
}
