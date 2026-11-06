// Copyright (c) 2020, 2023 The xterm.js authors. MIT license.
//! SIXEL protocol lifecycle, separate from actual canvas and image storage.
use super::{DecodeError, DecoderOptions, DecoderProperties, SixelDecoder, ADDON_PALETTE};
use crate::image::types::RgbaImage;
use std::fmt;
use xterm_core::parser::params::Params;

const PERMANENT_MEMORY_LIMIT: usize = 4_194_304;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SixelWarning {
    TooMuchData,
    Decoding(DecodeError),
}

impl fmt::Display for SixelWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooMuchData => f.write_str("SIXEL: too much data, aborting"),
            Self::Decoding(error) => write!(
                f,
                "SIXEL: error while decoding image - {}: {error}",
                error.js_name()
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SixelPut {
    Continue,
    Warning(SixelWarning),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SixelEndPlan {
    Handled,
    AdvanceCursor { height: i64 },
    Image { width: i64, height: i64 },
}

#[derive(Debug, PartialEq, Eq)]
pub enum SixelEnd {
    Handled,
    AdvanceCursor { height: i64 },
    Image(RgbaImage),
}

/// Loading is explicit because source hook/put calls before decoder readiness
/// are ignored, even if a decoder becomes ready during that same sequence.
#[derive(Default)]
pub struct SixelHandler {
    decoder: Option<SixelDecoder>,
    size: f64,
    aborted: bool,
}

impl SixelHandler {
    pub fn pending() -> Self {
        Self::default()
    }

    pub fn decoder_options(pixel_limit: f64, palette_limit: f64) -> DecoderOptions {
        DecoderOptions {
            memory_limit: Some(pixel_limit * 4.0),
            palette: ADDON_PALETTE.to_vec(),
            palette_limit,
            ..DecoderOptions::default()
        }
    }

    pub fn ready(pixel_limit: f64, palette_limit: f64) -> Result<Self, DecodeError> {
        Ok(Self {
            decoder: Some(SixelDecoder::new(Self::decoder_options(
                pixel_limit,
                palette_limit,
            ))?),
            ..Self::default()
        })
    }

    /// Called by the host's decoder-initialization completion. Do not replay a
    /// hook or input discarded while loading; the source does not replay it.
    pub fn complete_loading(&mut self, decoder: SixelDecoder) {
        self.decoder = Some(decoder);
    }
    pub fn is_ready(&self) -> bool {
        self.decoder.is_some()
    }
    pub fn size(&self) -> f64 {
        self.size
    }
    pub fn aborted(&self) -> bool {
        self.aborted
    }

    pub fn reset(&mut self, palette_limit: f64) {
        if self.begin_reset() {
            self.finish_reset(palette_limit);
        }
    }

    /// Release pixels and clear the palette before a host's fallible init
    /// argument conversion. Pending handlers never evaluate that argument.
    pub fn begin_reset(&mut self) -> bool {
        let Some(decoder) = &mut self.decoder else {
            return false;
        };
        decoder.release();
        decoder.clear_palette();
        true
    }

    /// Complete a started reset after releasing the borrow over user code.
    /// Reentry may have changed this decoder's state before init continues.
    pub fn finish_reset(&mut self, palette_limit: f64) {
        if let Some(decoder) = &mut self.decoder {
            let truncate = decoder.options.truncate;
            decoder.init(0, Some(&ADDON_PALETTE), palette_limit, truncate);
        }
    }

    /// Background lookup stays lazy: pending decoders and P2=1 never access it.
    pub fn hook<E>(
        &mut self,
        params: &Params,
        palette_limit: f64,
        background: impl FnOnce() -> Result<u32, E>,
    ) -> Result<(), E> {
        if self.begin_hook() {
            // The source reads the allocated backing slot, not active length.
            // A previous sequence's P2 may therefore survive a Params reset.
            let fill = if params.stored_value(1) == Some(1) {
                0
            } else {
                background()?
            };
            self.finish_hook(fill, palette_limit);
        }
        Ok(())
    }

    /// Commit size/abort changes before background lookup and palette coercion.
    pub fn begin_hook(&mut self) -> bool {
        self.size = 0.0;
        self.aborted = false;
        self.is_ready()
    }

    /// Initialize after host callbacks, without repeating the hook prologue.
    pub fn finish_hook(&mut self, fill: u32, palette_limit: f64) {
        if let Some(decoder) = &mut self.decoder {
            let truncate = decoder.options.truncate;
            decoder.init(fill, None, palette_limit, truncate);
        }
    }

    /// A warning is emitted by the host before `finish_abort` is called. This
    /// preserves source state if console.warn throws or reenters the addon.
    #[must_use]
    pub fn put(&mut self, data: &[u32], size_limit: f64) -> SixelPut {
        let Some(size) = self.begin_put(data.len()) else {
            return SixelPut::Continue;
        };
        if size > size_limit {
            return SixelPut::Warning(SixelWarning::TooMuchData);
        }
        self.decode_put(data)
    }

    /// Commit the packet size before a host's fallible limit comparison. A host
    /// must release this borrow before coercion, then either warn/finish_abort or
    /// call decode_put. Reentry may change the live decoder between these steps.
    pub fn begin_put(&mut self, length: usize) -> Option<f64> {
        if self.aborted || self.decoder.is_none() {
            return None;
        }
        self.size += length as f64;
        Some(self.size)
    }

    /// Continue a started packet without repeating the abort/readiness guard.
    /// Source coercion can reset or hook the handler before decoder.decode.
    #[must_use]
    pub fn decode_put(&mut self, data: &[u32]) -> SixelPut {
        let Some(decoder) = &mut self.decoder else {
            return SixelPut::Continue;
        };
        match decoder.decode(data) {
            Ok(()) => SixelPut::Continue,
            Err(error) => SixelPut::Warning(SixelWarning::Decoding(error)),
        }
    }

    pub fn mark_aborted(&mut self) {
        self.aborted = true;
    }

    pub fn release(&mut self) {
        if let Some(decoder) = &mut self.decoder {
            decoder.release();
        }
    }

    pub fn begin_release(&mut self) -> bool {
        if let Some(decoder) = &mut self.decoder {
            decoder.begin_release();
            true
        } else {
            false
        }
    }
    pub fn finish_release(&mut self, palette_limit: f64) {
        if let Some(decoder) = &mut self.decoder {
            decoder.finish_release(palette_limit);
        }
    }
    pub fn take_pending_release(&mut self) -> bool {
        self.decoder
            .as_mut()
            .is_some_and(SixelDecoder::take_pending_release)
    }
    pub fn clear_palette(&mut self) {
        if let Some(decoder) = &mut self.decoder {
            decoder.clear_palette();
        }
    }
    pub fn needs_release_after_copy(&self) -> Result<bool, DecodeError> {
        self.decoder.as_ref().map_or(Ok(false), |decoder| {
            decoder
                .memory_usage()
                .map(|usage| usage > PERMANENT_MEMORY_LIMIT)
        })
    }

    pub fn finish_abort(&mut self) {
        self.mark_aborted();
        self.release();
    }

    pub fn end_plan(&self, success: bool) -> SixelEndPlan {
        let Some(decoder) = &self.decoder else {
            return SixelEndPlan::Handled;
        };
        plan(decoder, success, self.aborted)
    }

    pub fn snapshot_image(&mut self) -> Result<Option<RgbaImage>, DecodeError> {
        self.decoder.as_mut().map(SixelDecoder::image).transpose()
    }

    /// Call after the canvas/ImageData copy, before inserting into storage.
    /// A context-less canvas may skip `snapshot_image`, as the source does.
    pub fn after_image_copy(&mut self) -> Result<(), DecodeError> {
        if let Some(decoder) = &mut self.decoder {
            release_large(decoder)?;
        }
        Ok(())
    }

    /// Native convenience path. Browser adapters use the plan/copy stages so
    /// no handler borrow is held across canvas methods or storage callbacks.
    pub fn end(&mut self, success: bool) -> Result<SixelEnd, DecodeError> {
        let Some(decoder) = &mut self.decoder else {
            return Ok(SixelEnd::Handled);
        };
        match plan(decoder, success, self.aborted) {
            SixelEndPlan::Handled => Ok(SixelEnd::Handled),
            SixelEndPlan::AdvanceCursor { height } => Ok(SixelEnd::AdvanceCursor { height }),
            SixelEndPlan::Image { .. } => {
                let image = decoder.image()?;
                release_large(decoder)?;
                Ok(SixelEnd::Image(image))
            }
        }
    }

    pub fn properties(&self) -> Result<Option<DecoderProperties>, DecodeError> {
        self.decoder
            .as_ref()
            .map(SixelDecoder::properties)
            .transpose()
    }
    pub fn palette(&self) -> Option<&[u32]> {
        self.decoder.as_ref().map(SixelDecoder::palette)
    }
}

fn plan(decoder: &SixelDecoder, success: bool, aborted: bool) -> SixelEndPlan {
    if aborted || !success {
        return SixelEndPlan::Handled;
    }
    let width = decoder.width();
    let height = decoder.height();
    if width == 0 || height == 0 {
        if height != 0 {
            SixelEndPlan::AdvanceCursor { height }
        } else {
            SixelEndPlan::Handled
        }
    } else {
        SixelEndPlan::Image { width, height }
    }
}

fn release_large(decoder: &mut SixelDecoder) -> Result<(), DecodeError> {
    if decoder.memory_usage()? > PERMANENT_MEMORY_LIMIT {
        decoder.release();
    }
    Ok(())
}
