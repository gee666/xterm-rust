// Copyright (c) 2019-2026 The xterm.js authors. All rights reserved. MIT license.
//! Retained browser links. JavaScript fields are read only at their source phase.
use super::{
    browser_linkifier::LinkCallback, Decorations, Link, Position, Range, RangePosition,
    UnderlineEvent,
};
use js_sys::{Array, Set};
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::MouseEvent;

#[wasm_bindgen(module = "/src/links/raw_link_bridge.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = linkProperty)]
    pub(crate) fn property(value: &JsValue, key: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = linkSet)]
    pub(super) fn set(value: &JsValue, key: &str, entry: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = linkCall)]
    pub(crate) fn call(
        value: &JsValue,
        receiver: &JsValue,
        args: &Array,
    ) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = linkMap)]
    pub(crate) fn map(value: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = linkAdd)]
    fn add(a: &JsValue, b: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = linkMultiply)]
    fn multiply(a: &JsValue, b: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = linkSubtract)]
    fn subtract(a: &JsValue, b: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = linkLessEqual)]
    pub(super) fn less_equal(a: &JsValue, b: &JsValue) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = linkLess)]
    fn less(a: &JsValue, b: &JsValue) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = linkIncrement)]
    fn increment(value: &JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Clone)]
pub struct NativeBrowserLink {
    pub data: Rc<Link>,
    pub decorations: Option<Decorations>,
    pub activate: LinkCallback,
    pub hover: Option<LinkCallback>,
    pub leave: Option<LinkCallback>,
    pub dispose: Option<super::browser_linkifier::LinkDisposal>,
}

#[derive(Clone)]
pub enum BrowserLink {
    Native(NativeBrowserLink),
    JavaScript(JsValue),
}
impl BrowserLink {
    fn endpoint(&self, end: &str, axis: &str) -> Result<JsValue, JsValue> {
        match self {
            Self::Native(link) => {
                let point = if end == "start" {
                    link.data.range.start
                } else {
                    link.data.range.end
                };
                Ok(JsValue::from_f64(
                    if axis == "x" { point.x } else { point.y } as f64,
                ))
            }
            Self::JavaScript(link) => property(&property(&property(link, "range")?, end)?, axis),
        }
    }
    pub fn contains(&self, position: Position, cols: usize) -> Result<bool, JsValue> {
        if let Self::Native(link) = self {
            return Ok(link.data.range.contains(position, cols));
        }
        let current = JsValue::from_f64(position.y as f64 * cols as f64 + position.x as f64);
        let cols = JsValue::from_f64(cols as f64);
        // The complete upper operand is evaluated even when lower > current.
        let lower = add(
            &multiply(&self.endpoint("start", "y")?, &cols)?,
            &self.endpoint("start", "x")?,
        )?;
        let upper = add(
            &multiply(&self.endpoint("end", "y")?, &cols)?,
            &self.endpoint("end", "x")?,
        )?;
        Ok(less_equal(&lower, &current)? && less_equal(&current, &upper)?)
    }
    pub(super) fn equals(&self, other: &Self) -> Result<bool, JsValue> {
        if self.text()? != other.text()? {
            return Ok(false);
        }
        for (end, axis) in [("start", "x"), ("start", "y"), ("end", "x"), ("end", "y")] {
            if self.endpoint(end, axis)? != other.endpoint(end, axis)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn text(&self) -> Result<JsValue, JsValue> {
        match self {
            Self::Native(link) => Ok(crate::utf16::to_js_string(&link.data.text).into()),
            Self::JavaScript(link) => property(link, "text"),
        }
    }
    pub(super) fn decorations(&self) -> Result<(JsValue, JsValue), JsValue> {
        match self {
            Self::Native(link) => {
                let decorations = link.decorations.unwrap_or_default();
                Ok((
                    decorations.underline.into(),
                    decorations.pointer_cursor.into(),
                ))
            }
            Self::JavaScript(link) => {
                // Source reads decorations separately for each conditional and field.
                let read = |key| {
                    if property(link, "decorations")?.is_undefined() {
                        Ok(JsValue::TRUE)
                    } else {
                        property(&property(link, "decorations")?, key)
                    }
                };
                Ok((read("underline")?, read("pointerCursor")?))
            }
        }
    }
    pub(super) fn invoke(&self, name: &str, event: &MouseEvent) -> Result<(), JsValue> {
        match self {
            Self::Native(link) => {
                let callback = match name {
                    "activate" => Some(&link.activate),
                    "hover" => link.hover.as_ref(),
                    "leave" => link.leave.as_ref(),
                    _ => unreachable!("fixed link callback names"),
                };
                if let Some(callback) = callback {
                    callback(event, &link.data)?;
                }
            }
            Self::JavaScript(link) => {
                if name == "activate" || property(link, name)?.is_truthy() {
                    let callback = property(link, name)?;
                    let text = property(link, "text")?;
                    call(&callback, link, &Array::of2(event, &text))?;
                }
            }
        }
        Ok(())
    }
    pub(super) fn dispose(&self) -> Result<(), JsValue> {
        match self {
            Self::Native(link) => {
                if let Some(dispose) = &link.dispose {
                    dispose()?;
                }
            }
            Self::JavaScript(link) => {
                if property(link, "dispose")?.is_truthy() {
                    call(&property(link, "dispose")?, link, &Array::new())?;
                }
            }
        }
        Ok(())
    }
    pub(super) fn keep_nonintersecting(
        &self,
        y: usize,
        cols: usize,
        occupied: &Set,
    ) -> Result<bool, JsValue> {
        let y = JsValue::from_f64(y as f64);
        let mut x = if less(&self.endpoint("start", "y")?, &y)? {
            0.into()
        } else {
            self.endpoint("start", "x")?
        };
        let end = if less(&y, &self.endpoint("end", "y")?)? {
            JsValue::from_f64(cols as f64)
        } else {
            self.endpoint("end", "x")?
        };
        while less_equal(&x, &end)? {
            if occupied.has(&x) {
                return Ok(false);
            }
            occupied.add(&x);
            x = increment(&x)?;
        }
        Ok(true)
    }
    pub(super) fn invalidated(&self, start: f64, end: f64) -> Result<bool, JsValue> {
        Ok(less_equal(&start.into(), &self.endpoint("start", "y")?)?
            && less_equal(&self.endpoint("end", "y")?, &end.into())?)
    }
    pub(super) fn underline_event(
        &self,
        cols: usize,
        offset: usize,
    ) -> Result<UnderlineEvent, JsValue> {
        match self {
            Self::Native(link) => Ok(link.data.underline_event(cols, offset)),
            Self::JavaScript(link) => {
                // _fireUnderlineEvent holds the range, not the individual endpoints.
                let range = property(link, "range")?;
                let point = |end, axis| property(&property(&range, end)?, axis);
                let one = JsValue::from_f64(1.0);
                let offset = JsValue::from_f64(offset as f64);
                Ok(UnderlineEvent {
                    x1: coordinate(&subtract(&point("start", "x")?, &one)?)? as i64,
                    y1: coordinate(&subtract(&subtract(&point("start", "y")?, &offset)?, &one)?)?
                        as i64,
                    x2: coordinate(&point("end", "x")?)? as u64,
                    y2: coordinate(&subtract(&subtract(&point("end", "y")?, &offset)?, &one)?)?
                        as i64,
                    cols,
                })
            }
        }
    }
    /// Selection samples the range once, without observing the text getter.
    pub fn range(&self) -> Result<Range, JsValue> {
        match self {
            Self::Native(link) => Ok(link.data.range),
            Self::JavaScript(link) => {
                let range = property(link, "range")?;
                let point = |end| -> Result<RangePosition, JsValue> {
                    let point = property(&range, end)?;
                    Ok(RangePosition {
                        x: coordinate(&property(&point, "x")?)? as u64,
                        y: coordinate(&property(&point, "y")?)? as u64,
                    })
                };
                Ok(Range {
                    start: point("start")?,
                    end: point("end")?,
                })
            }
        }
    }
    /// Diagnostics read a fresh value; the controller never uses this snapshot.
    pub fn snapshot(&self) -> Result<Link, JsValue> {
        match self {
            Self::Native(link) => Ok((*link.data).clone()),
            Self::JavaScript(_) => {
                let text = self.text()?;
                if !text.is_string() {
                    return Err(js_sys::TypeError::new("Link text is not a string").into());
                }
                Ok(Link {
                    text: crate::utf16::from_js_string(&text.unchecked_into()),
                    range: self.range()?,
                })
            }
        }
    }
}
fn coordinate(value: &JsValue) -> Result<f64, JsValue> {
    value
        .as_f64()
        .filter(|v| v.is_finite() && v.fract() == 0.0 && v.abs() <= f64::from(u32::MAX))
        .ok_or_else(|| {
            js_sys::RangeError::new("Link drawing coordinates must be finite 32-bit integers")
                .into()
        })
}
