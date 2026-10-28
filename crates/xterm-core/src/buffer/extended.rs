// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::constants::{attributes, ext_flags, underline_style};
use std::{any::Any, cell::RefCell, fmt, rc::Rc};

#[derive(Default)]
struct Data {
    raw: u32,
    url_id: u64,
    payload: Option<Rc<dyn Any>>,
}

/// Packed underline attributes and hyperlink identity.
///
/// `share()` retains JS object identity for buffer storage and load operations.
/// `clone()` matches upstream's explicit clone: copies packed data and URL but
/// drops payload. Payload is an opaque shared Rust value corresponding to Object.
#[derive(Default)]
pub struct ExtendedAttrs {
    data: Rc<RefCell<Data>>,
}

impl ExtendedAttrs {
    pub fn new(ext: u32, url_id: u64) -> Self {
        Self {
            data: Rc::new(RefCell::new(Data {
                raw: ext,
                url_id,
                payload: None,
            })),
        }
    }

    /// Share the same mutable object rather than invoking upstream `clone()`.
    pub fn share(&self) -> Self {
        Self {
            data: Rc::clone(&self.data),
        }
    }

    pub fn same_identity(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data)
    }

    /// Valid internal OSC IDs are nonnegative integers. Keep them separate from
    /// packed u32 data so generated JavaScript IDs through 2^53 retain identity.
    pub fn url_id(&self) -> u64 {
        self.data.borrow().url_id
    }
    pub fn set_url_id(&mut self, value: u64) {
        self.data.borrow_mut().url_id = value;
    }
    pub fn payload(&self) -> Option<Rc<dyn Any>> {
        self.data.borrow().payload.clone()
    }
    pub fn set_payload(&mut self, value: Option<Rc<dyn Any>>) {
        // Native payload destruction can run user code. Publish the replacement
        // and release the borrow before dropping the previous owner.
        let previous = std::mem::replace(&mut self.data.borrow_mut().payload, value);
        drop(previous);
    }

    /// Stored word before hyperlink-driven underline presentation overrides.
    pub fn raw_ext(&self) -> u32 { self.data.borrow().raw }

    pub fn ext(&self) -> u32 {
        let data = self.data.borrow();
        if data.url_id != 0 {
            (data.raw & !ext_flags::UNDERLINE_STYLE) | (underline_style::DASHED << 26)
        } else {
            data.raw
        }
    }

    pub fn set_ext(&mut self, value: u32) {
        self.data.borrow_mut().raw = value;
    }

    pub fn underline_style(&self) -> u32 {
        let data = self.data.borrow();
        if data.url_id != 0 {
            underline_style::DASHED
        } else {
            (data.raw & ext_flags::UNDERLINE_STYLE) >> 26
        }
    }

    pub fn set_underline_style(&mut self, value: u32) {
        let mut data = self.data.borrow_mut();
        data.raw =
            (data.raw & !ext_flags::UNDERLINE_STYLE) | ((value << 26) & ext_flags::UNDERLINE_STYLE);
    }

    pub fn underline_color(&self) -> u32 {
        self.data.borrow().raw & (attributes::CM_MASK | attributes::RGB_MASK)
    }

    pub fn set_underline_color(&mut self, value: u32) {
        const MASK: u32 = attributes::CM_MASK | attributes::RGB_MASK;
        let mut data = self.data.borrow_mut();
        data.raw = (data.raw & !MASK) | (value & MASK);
    }

    pub fn underline_variant_offset(&self) -> u32 {
        // Unsigned extraction equals the source's sign extension followed by XOR.
        (self.data.borrow().raw & ext_flags::VARIANT_OFFSET) >> 29
    }

    pub fn set_underline_variant_offset(&mut self, value: u32) {
        let mut data = self.data.borrow_mut();
        data.raw =
            (data.raw & !ext_flags::VARIANT_OFFSET) | ((value << 29) & ext_flags::VARIANT_OFFSET);
    }

    pub fn is_empty(&self) -> bool {
        let data = self.data.borrow();
        data.url_id == 0 && (data.raw & ext_flags::UNDERLINE_STYLE) == 0 && data.payload.is_none()
    }
}

impl Clone for ExtendedAttrs {
    fn clone(&self) -> Self {
        let data = self.data.borrow();
        Self::new(data.raw, data.url_id)
    }
}

impl fmt::Debug for ExtendedAttrs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = self.data.borrow();
        f.debug_struct("ExtendedAttrs")
            .field("raw", &data.raw)
            .field("url_id", &data.url_id)
            .field("has_payload", &data.payload.is_some())
            .finish()
    }
}
