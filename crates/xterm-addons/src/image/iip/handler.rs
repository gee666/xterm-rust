// Copyright (c) 2023 The xterm.js authors. MIT license.
//! OSC1337 streaming transfer state. Real image decoding is an explicit host effect.
use super::geometry::{IipGeometry, SizeRequest};
use super::reports::cell_size_report;
use super::types::{
    default_header, merge_header, number_truthy, truthy, DisplayRequest, EndPreparation, IipEffect,
    IipError, IipOptions, PendingEnd, PendingKind, DECODER_INITIAL_BYTES, DECODER_KEEP_BYTES,
};
use crate::image::base64::{DecodeStatus, PayloadDecoder};
use crate::image::header::{
    HeaderFields, HeaderParser, HeaderState, HeaderValue, ParseOutcome, SequenceType,
};
use crate::image::metrics::{image_type, ImageType};
use crate::image::NameDecodePolicy;

pub struct IipHandler {
    options: IipOptions,
    owner: std::rc::Rc<()>,
    aborted: bool,
    parser: HeaderParser,
    header: HeaderFields,
    decoder: PayloadDecoder,
    multipart: bool,
    abort_multipart: bool,
}
impl IipHandler {
    pub fn new(options: IipOptions, policy: NameDecodePolicy) -> Result<Self, IipError> {
        let encoded_limit = options
            .size_limit
            .checked_mul(4)
            .and_then(|n| n.checked_add(2))
            .map(|n| n / 3)
            .ok_or(crate::image::base64::InvalidSettings)?;
        let decoder = PayloadDecoder::new(
            DECODER_KEEP_BYTES,
            encoded_limit,
            DECODER_INITIAL_BYTES.min(encoded_limit),
        )?;
        Ok(Self {
            options,
            owner: std::rc::Rc::new(()),
            aborted: false,
            parser: HeaderParser::new(policy),
            header: default_header(),
            decoder,
            multipart: false,
            abort_multipart: false,
        })
    }
    /// Source reset releases decoder state but intentionally retains multipart flags/header.
    pub fn reset(&mut self) {
        self.parser.reset();
        self.decoder.release();
    }
    pub fn start(&mut self) {
        self.aborted = false;
        self.parser.reset();
    }
    pub fn header_parser(&self) -> &HeaderParser {
        &self.parser
    }
    pub fn active_header(&self) -> &HeaderFields {
        &self.header
    }
    pub fn is_aborted(&self) -> bool {
        self.aborted
    }
    pub fn is_multipart(&self) -> bool {
        self.multipart
    }
    pub fn is_multipart_aborted(&self) -> bool {
        self.abort_multipart
    }
    pub fn loaded_bytes(&self) -> usize {
        self.decoder.loaded_bytes()
    }

    pub fn put(&mut self, data: &[u32], start: usize, end: usize) -> Result<(), IipError> {
        if start > end || end > data.len() {
            return Err(crate::image::header::HeaderRangeError.into());
        }
        if self.aborted {
            return Ok(());
        }
        if self.parser.state == HeaderState::End {
            if self.decoder.put(&data[start..end])? != DecodeStatus::Ok {
                self.decoder.release();
                self.aborted = true;
            }
            return Ok(());
        }
        match self.parser.parse(data, start, end)? {
            ParseOutcome::Rejected => self.aborted = true,
            ParseOutcome::NeedMore => (),
            ParseOutcome::Payload(position) => {
                if is_type(&self.parser.fields, SequenceType::File) {
                    if self.multipart {
                        self.multipart = false;
                        self.abort_multipart = false;
                        self.decoder.release();
                    }
                    self.header = merge_header(&self.parser.fields);
                    if !self.header.get_ascii("inline").is_some_and(truthy) {
                        self.aborted = true;
                        return Ok(());
                    }
                    self.decoder.init(None, None)?;
                } else if self.abort_multipart {
                    self.aborted = true;
                    return Ok(());
                }
                if self.decoder.put(&data[position..end])? != DecodeStatus::Ok {
                    self.decoder.release();
                    self.aborted = true;
                    if self.multipart {
                        self.abort_multipart = true;
                    }
                }
            }
        }
        Ok(())
    }
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
        if self.aborted {
            return Ok(EndPreparation::Complete(Vec::new()));
        }
        if self.parser.state != HeaderState::End && self.parser.end() == ParseOutcome::Rejected {
            return Ok(EndPreparation::Complete(Vec::new()));
        }
        if is_type(&self.parser.fields, SequenceType::FilePart) {
            return Ok(EndPreparation::Complete(Vec::new()));
        }
        if is_type(&self.parser.fields, SequenceType::ReportCellSize) {
            return Ok(EndPreparation::Geometry(PendingEnd {
                owner: self.owner.clone(),
                kind: PendingKind::Report,
            }));
        }
        if is_type(&self.parser.fields, SequenceType::MultipartFile) {
            self.header = merge_header(&self.parser.fields);
            self.multipart = true;
            self.abort_multipart = false;
            self.decoder.release();
            self.decoder.init(None, None)?;
            return Ok(EndPreparation::Complete(Vec::new()));
        }
        if is_type(&self.parser.fields, SequenceType::FileEnd) {
            if !self.multipart {
                return Ok(EndPreparation::Complete(Vec::new()));
            }
            self.multipart = false;
            if self.abort_multipart || !is_type(&self.header, SequenceType::MultipartFile) {
                return Ok(EndPreparation::Complete(Vec::new()));
            }
        }
        if !success {
            self.decoder.release();
            return Ok(EndPreparation::Complete(Vec::new()));
        }
        if self.decoder.end()? != DecodeStatus::Ok {
            self.decoder.release();
            return Ok(EndPreparation::Complete(vec![IipEffect::Warning(
                "IIP: error during BASE64 decoding".to_owned(),
            )]));
        }
        let metrics = image_type(self.decoder.data());
        if metrics.mime == ImageType::Unsupported {
            self.decoder.release();
            return Ok(EndPreparation::Complete(vec![IipEffect::Warning(
                "IIP: unsupported image type".to_owned(),
            )]));
        }
        let width = f64::from(metrics.width);
        let height = f64::from(metrics.height);
        if !number_truthy(width)
            || !number_truthy(height)
            || width * height >= self.options.pixel_limit
            || self.options.pixel_limit.is_nan()
        {
            self.decoder.release();
            return Ok(EndPreparation::Complete(vec![IipEffect::Warning(format!(
                "IIP: image dimension issue {}x{}",
                metrics.width, metrics.height
            ))]));
        }
        Ok(EndPreparation::Geometry(PendingEnd {
            owner: self.owner.clone(),
            kind: PendingKind::Resize(metrics),
        }))
    }
    pub fn finish_geometry(
        &mut self,
        token: PendingEnd,
        geometry: &IipGeometry,
    ) -> Result<Vec<IipEffect>, IipError> {
        if !std::rc::Rc::ptr_eq(&token.owner, &self.owner) {
            return Err(IipError::ForeignEndToken);
        }
        let metrics = match token.kind {
            PendingKind::Report => return Ok(vec![IipEffect::Report(cell_size_report(geometry))]),
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
        if !number_truthy(target.width)
            || !number_truthy(target.height)
            || (target.width * target.height).partial_cmp(&self.options.pixel_limit)
                != Some(std::cmp::Ordering::Less)
        {
            self.decoder.release();
            return Ok(Vec::new());
        }
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(self.decoder.data().len())?;
        bytes.extend_from_slice(self.decoder.data());
        self.decoder.release();
        Ok(vec![IipEffect::Display(DisplayRequest {
            bytes,
            source: metrics,
            target,
        })])
    }
    fn size_request(&self) -> Result<SizeRequest, IipError> {
        let text = |field| match self.header.get_ascii(field) {
            Some(HeaderValue::Text(units)) => Ok(units.clone()),
            _ => Err(IipError::HeaderField(field)),
        };
        let preserve_aspect_ratio = match self.header.get_ascii("preserveAspectRatio") {
            Some(HeaderValue::Number(value)) => *value,
            _ => return Err(IipError::HeaderField("preserveAspectRatio")),
        };
        Ok(SizeRequest {
            width: text("width")?,
            height: text("height")?,
            preserve_aspect_ratio,
        })
    }
    /// Only the createImageBitmap rejection is caught upstream. Synchronous QOI
    /// decoding/ImageData failures must propagate through the caller instead.
    pub fn bitmap_failure(
        request: &DisplayRequest,
        cause: std::rc::Rc<dyn std::error::Error>,
    ) -> IipEffect {
        IipEffect::HostWarning {
            message: format!(
                "IIP: decoding error {} {}x{}",
                request.source.mime.mime(),
                request.source.width,
                request.source.height
            ),
            cause,
        }
    }
}
fn is_type(fields: &HeaderFields, kind: SequenceType) -> bool {
    fields.get_ascii("type") == Some(&HeaderValue::Number(f64::from(kind as u8)))
}
