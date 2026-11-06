// Copyright (c) 2023 The xterm.js authors. MIT license.
//! End stages release controller borrows before geometry and raw-limit callbacks.
use super::super::types::{
    merge_header, number_truthy, DisplayRequest, EndPreparation, IipEffect, IipError,
    OriginalPixelCheck, PendingEnd, PendingKind, PixelEndPreparation, PixelResizePreparation,
    ResizedPixelCheck,
};
use super::super::{geometry::IipGeometry, reports::cell_size_report};
use super::{is_type, IipHandler};
use crate::image::metrics::ImageMetrics;
use crate::image::{
    base64::DecodeStatus,
    header::{HeaderState, ParseOutcome, SequenceType},
    metrics::{image_type, ImageType},
};
impl IipHandler {
    /// No effect means the sequence was consumed with no image/reply, as upstream.
    /// Display effects must finish real host decode/insertion before resolving OSC.
    pub fn end(
        &mut self,
        success: bool,
        geometry: &IipGeometry,
    ) -> Result<Vec<IipEffect>, IipError> {
        match self.prepare_end(success)? {
            EndPreparation::Complete(effects) => Ok(effects),
            EndPreparation::Geometry(token) => self.finish_geometry(token, geometry),
        }
    }
    /// Return before any foreign geometry getter. Adapters must release their
    /// controller borrow, call the fallible host, then complete the owned token.
    pub fn prepare_end(&mut self, success: bool) -> Result<EndPreparation, IipError> {
        match self.prepare_end_pixels(success)? {
            PixelEndPreparation::End(end) => Ok(end),
            PixelEndPreparation::Check(check) => {
                let accepted = check.area() < self.options.pixel_limit;
                self.finish_original_pixels(check, accepted)
            }
        }
    }
    /// Stop before comparing the original image area with a host-owned limit.
    /// A throwing comparison leaves the live payload retained.
    pub fn prepare_end_pixels(&mut self, success: bool) -> Result<PixelEndPreparation, IipError> {
        if self.aborted {
            return Ok(PixelEndPreparation::End(EndPreparation::Complete(
                Vec::new(),
            )));
        }
        if self.parser.state != HeaderState::End && self.parser.end() == ParseOutcome::Rejected {
            return Ok(PixelEndPreparation::End(EndPreparation::Complete(
                Vec::new(),
            )));
        }
        if is_type(&self.parser.fields, SequenceType::FilePart) {
            return Ok(PixelEndPreparation::End(EndPreparation::Complete(
                Vec::new(),
            )));
        }
        if is_type(&self.parser.fields, SequenceType::ReportCellSize) {
            return Ok(PixelEndPreparation::End(EndPreparation::Geometry(
                PendingEnd {
                    owner: self.owner.clone(),
                    kind: PendingKind::Report,
                },
            )));
        }
        if is_type(&self.parser.fields, SequenceType::MultipartFile) {
            self.header = merge_header(&self.parser.fields);
            self.multipart = true;
            self.abort_multipart = false;
            self.decoder.release();
            self.decoder.init()?;
            return Ok(PixelEndPreparation::End(EndPreparation::Complete(
                Vec::new(),
            )));
        }
        if is_type(&self.parser.fields, SequenceType::FileEnd) {
            if !self.multipart {
                return Ok(PixelEndPreparation::End(EndPreparation::Complete(
                    Vec::new(),
                )));
            }
            self.multipart = false;
            if self.abort_multipart || !is_type(&self.header, SequenceType::MultipartFile) {
                return Ok(PixelEndPreparation::End(EndPreparation::Complete(
                    Vec::new(),
                )));
            }
        }
        if !success {
            self.decoder.release();
            return Ok(PixelEndPreparation::End(EndPreparation::Complete(
                Vec::new(),
            )));
        }
        if self.decoder.end()? != DecodeStatus::Ok {
            return Ok(PixelEndPreparation::End(EndPreparation::Complete(vec![
                IipEffect::Warning("IIP: error during BASE64 decoding".to_owned()),
            ])));
        }
        let metrics = image_type(self.decoder.data());
        if metrics.mime == ImageType::Unsupported {
            return Ok(PixelEndPreparation::End(EndPreparation::Complete(vec![
                IipEffect::Warning("IIP: unsupported image type".to_owned()),
            ])));
        }
        if metrics.width == 0 || metrics.height == 0 {
            return Ok(PixelEndPreparation::End(EndPreparation::Complete(vec![
                IipEffect::Warning(dimension_warning(metrics)),
            ])));
        }
        Ok(PixelEndPreparation::Check(OriginalPixelCheck {
            owner: self.owner.clone(),
            source: metrics,
        }))
    }
    /// Complete after the host comparison with no controller borrow held.
    pub fn finish_original_pixels(
        &mut self,
        check: OriginalPixelCheck,
        accepted: bool,
    ) -> Result<EndPreparation, IipError> {
        if !std::rc::Rc::ptr_eq(&check.owner, &self.owner) {
            return Err(IipError::ForeignEndToken);
        }
        if !accepted {
            return Ok(EndPreparation::Complete(vec![IipEffect::Warning(
                dimension_warning(check.source),
            )]));
        }
        Ok(EndPreparation::Geometry(PendingEnd {
            owner: self.owner.clone(),
            kind: PendingKind::Resize(check.source),
        }))
    }
    pub fn finish_geometry(
        &mut self,
        token: PendingEnd,
        geometry: &IipGeometry,
    ) -> Result<Vec<IipEffect>, IipError> {
        match self.prepare_resize_pixels(token, geometry)? {
            PixelResizePreparation::Complete(effects) => Ok(effects),
            PixelResizePreparation::Check(check) => {
                let accepted = check.area() < self.options.pixel_limit;
                self.finish_resized_pixels(check, accepted)
            }
        }
    }
    /// Geometry and header evaluation finish before the second limit read.
    pub fn prepare_resize_pixels(
        &mut self,
        token: PendingEnd,
        geometry: &IipGeometry,
    ) -> Result<PixelResizePreparation, IipError> {
        if !std::rc::Rc::ptr_eq(&token.owner, &self.owner) {
            return Err(IipError::ForeignEndToken);
        }
        let metrics = match token.kind {
            PendingKind::Report => {
                return Ok(PixelResizePreparation::Complete(vec![IipEffect::Report(
                    cell_size_report(geometry),
                )]))
            }
            PendingKind::Resize(metrics) => metrics,
        };
        // Reentrant host code may reset or start another transfer. Source reads
        // current header/decoder data after its geometry getters, not a stale copy.
        let request = self.size_request()?;
        let mut target = geometry.resize(
            f64::from(metrics.width),
            f64::from(metrics.height),
            &request,
        );
        target.width = target.width.floor();
        target.height = target.height.floor();
        if !number_truthy(target.width) || !number_truthy(target.height) {
            self.decoder.release();
            return Ok(PixelResizePreparation::Complete(Vec::new()));
        }
        Ok(PixelResizePreparation::Check(ResizedPixelCheck {
            owner: self.owner.clone(),
            source: metrics,
            target,
        }))
    }
    /// The comparison may have reset or started another transfer. Copy the
    /// current decoder only now, while dimensions remain the captured locals.
    pub fn finish_resized_pixels(
        &mut self,
        check: ResizedPixelCheck,
        accepted: bool,
    ) -> Result<Vec<IipEffect>, IipError> {
        if !std::rc::Rc::ptr_eq(&check.owner, &self.owner) {
            return Err(IipError::ForeignEndToken);
        }
        if !accepted {
            self.decoder.release();
            return Ok(Vec::new());
        }
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(self.decoder.data().len())?;
        bytes.extend_from_slice(self.decoder.data());
        Ok(vec![IipEffect::Display(DisplayRequest {
            bytes,
            source: check.source,
            target: check.target,
        })])
    }
}
fn dimension_warning(metrics: ImageMetrics) -> String {
    format!(
        "IIP: image dimension issue {}x{}",
        metrics.width, metrics.height
    )
}
