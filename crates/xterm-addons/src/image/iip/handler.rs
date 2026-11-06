// Copyright (c) 2023 The xterm.js authors. MIT license.
//! OSC1337 streaming transfer state. Real image decoding is an explicit host effect.
mod end;
use super::decoder::IipDecoder;
use super::geometry::SizeRequest;
use super::types::{
    default_header, merge_header, truthy, DisplayRequest, IipEffect, IipError, IipOptions,
};
use crate::image::base64::DecodeStatus;
use crate::image::header::{
    HeaderFields, HeaderParser, HeaderState, HeaderValue, ParseOutcome, SequenceType,
};
use crate::image::NameDecodePolicy;

pub struct IipHandler {
    options: IipOptions,
    owner: std::rc::Rc<()>,
    aborted: bool,
    parser: HeaderParser,
    header: HeaderFields,
    decoder: IipDecoder,
    multipart: bool,
    abort_multipart: bool,
}
impl IipHandler {
    pub fn new(options: IipOptions, policy: NameDecodePolicy) -> Result<Self, IipError> {
        // Source rounds the encoded limit after floating-point arithmetic.
        // In particular, fractional byte limits must not be truncated first.
        Self::new_with_encoded_limit(options, policy, (options.size_limit * 4.0 / 3.0).ceil())
    }
    /// Accept the encoded-byte cap already evaluated by a browser host. Raw JS
    /// conversion and its exceptions must happen before borrowing the handler.
    /// NaN/negative settings construct as upstream but fail safely on decoder
    /// initialization; compatibility with those decoder paths is not claimed.
    pub fn new_with_encoded_limit(
        options: IipOptions,
        policy: NameDecodePolicy,
        encoded_limit: f64,
    ) -> Result<Self, IipError> {
        let decoder = IipDecoder::new(encoded_limit)?;
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
    /// Call after successful ImageData/QOI preparation or Blob construction,
    /// before creating the canvas or calling createImageBitmap. Synchronous
    /// failures before that boundary retain payload state, as upstream does.
    /// Release the live decoder even if a host callback started another transfer.
    pub fn release_payload(&mut self) {
        self.decoder.release();
    }

    /// Deliver an end warning without holding the controller borrow across host
    /// code. A thrown/failed callback retains payload state. A successful callback
    /// releases the live decoder, including a transfer started by that callback.
    pub fn report_warning<E>(
        handler: &std::cell::RefCell<Self>,
        message: &str,
        report: impl FnOnce(&str) -> Result<(), E>,
    ) -> Result<(), E> {
        report(message)?;
        handler.borrow_mut().release_payload();
        Ok(())
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
                    self.decoder.init()?;
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
