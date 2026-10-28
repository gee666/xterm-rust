// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.
use super::{OptionError, OptionValue};
use std::any::Any;
type PropertyGetter = dyn Fn(&dyn Any, &str) -> Result<OptionValue, OptionError>;
type Equality = dyn Fn(&OptionObject, &OptionObject) -> bool;

/// Relational/arithmetic JS operators differ from ToNumber for BigInt. A host
/// supplies these operations for opaque values, without coupling core to JS.
pub trait NumericSemantics {
    fn add_left(&self, _rhs: &OptionValue) -> Option<Result<OptionValue, OptionError>> {
        None
    }
    fn add_right(&self, _lhs: &OptionValue) -> Option<Result<OptionValue, OptionError>> {
        None
    }
    /// Optional JS addition/default-hint conversion. None uses the ordinary
    /// object's Number-hint ToPrimitive contract, not a string-hint fallback.
    fn concat_utf16(&self) -> Option<Result<Vec<u16>, OptionError>> {
        None
    }
    fn less_than(&self, rhs: f64) -> Result<bool, OptionError>;
    fn less_equal(&self, rhs: f64) -> Result<bool, OptionError>;
    fn multiply(&self, rhs: f64) -> Result<f64, OptionError>;
}
use std::fmt;
use std::rc::Rc;

/// An adapter-owned object. Clones preserve identity and ownership. A browser
/// adapter can retain a DOM document, logger, callback, or structured JS object.
/// Numeric/string coercion is supplied by that adapter, not guessed by the core.
pub struct OptionObject {
    payload: Box<dyn Any>,
    backing: Option<Box<dyn Any>>,
    to_primitive: Box<dyn Fn(CoercionHint) -> Result<OptionValue, OptionError>>,
    property_getter: Box<PropertyGetter>,
    truthy: bool,
    equal: Option<Box<Equality>>,
    numeric: Option<Rc<dyn NumericSemantics>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoercionHint {
    Number,
    String,
}

/// Host conversion failures retain lossless JS error text. The host adapter
/// invokes JS ToPrimitive using the supplied hint and forwards thrown errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoercionError(pub Vec<u16>);
impl CoercionError {
    pub fn new(message: &str) -> Self {
        Self(message.encode_utf16().collect())
    }
}
impl fmt::Display for CoercionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&String::from_utf16_lossy(&self.0))
    }
}
impl std::error::Error for CoercionError {}

impl OptionObject {
    pub fn new<T: Any>(
        payload: T,
        to_primitive: impl Fn(CoercionHint) -> Result<OptionValue, CoercionError> + 'static,
    ) -> Rc<Self> {
        Self::with_fallible_properties(
            payload,
            move |hint| to_primitive(hint).map_err(Into::into),
            |_, _| Ok(OptionValue::Undefined),
        )
    }

    /// Supply property access for structured payloads or JS getters. The getter
    /// can retain interior mutability and propagate host exceptions losslessly.
    pub fn with_properties<T: Any>(
        payload: T,
        to_primitive: impl Fn(CoercionHint) -> Result<OptionValue, CoercionError> + 'static,
        get: impl Fn(&T, &str) -> Result<OptionValue, CoercionError> + 'static,
    ) -> Rc<Self> {
        Self::with_fallible_properties(
            payload,
            move |hint| to_primitive(hint).map_err(Into::into),
            move |payload, key| get(payload, key).map_err(Into::into),
        )
    }

    /// Full host callback errors can retain the original exception via
    /// OptionError::Dispatch and EmitError::callback, without string conversion.
    pub fn with_fallible_properties<T: Any>(
        payload: T,
        to_primitive: impl Fn(CoercionHint) -> Result<OptionValue, OptionError> + 'static,
        get: impl Fn(&T, &str) -> Result<OptionValue, OptionError> + 'static,
    ) -> Rc<Self> {
        Rc::new(Self {
            payload: Box::new(payload),
            backing: None,
            to_primitive: Box::new(to_primitive),
            truthy: true,
            equal: None,
            numeric: None,
            property_getter: Box::new(move |payload, key| {
                // This object owns exactly the T boxed by this constructor.
                get(
                    payload
                        .downcast_ref::<T>()
                        .expect("property getter payload type"),
                    key,
                )
            }),
        })
    }

    /// Represent host primitives not natively modelled by OptionValue, such as
    /// JS Symbol/BigInt, without losing truthiness or strict equality. Equality
    /// must be a pure operation, not an observable user callback.
    pub fn with_value_semantics<T: Any>(
        payload: T,
        to_primitive: impl Fn(CoercionHint) -> Result<OptionValue, OptionError> + 'static,
        get: impl Fn(&T, &str) -> Result<OptionValue, OptionError> + 'static,
        truthy: bool,
        equal: impl Fn(&T, &T) -> bool + 'static,
    ) -> Rc<Self> {
        Self::make_value(payload, to_primitive, get, truthy, equal, None)
    }

    pub fn with_numeric_value_semantics<T: Any>(
        payload: T,
        to_primitive: impl Fn(CoercionHint) -> Result<OptionValue, OptionError> + 'static,
        get: impl Fn(&T, &str) -> Result<OptionValue, OptionError> + 'static,
        truthy: bool,
        equal: impl Fn(&T, &T) -> bool + 'static,
        numeric: Rc<dyn NumericSemantics>,
    ) -> Rc<Self> {
        Self::make_value(payload, to_primitive, get, truthy, equal, Some(numeric))
    }

    fn make_value<T: Any>(
        payload: T,
        to_primitive: impl Fn(CoercionHint) -> Result<OptionValue, OptionError> + 'static,
        get: impl Fn(&T, &str) -> Result<OptionValue, OptionError> + 'static,
        truthy: bool,
        equal: impl Fn(&T, &T) -> bool + 'static,
        numeric: Option<Rc<dyn NumericSemantics>>,
    ) -> Rc<Self> {
        Rc::new(Self {
            payload: Box::new(payload),
            backing: None,
            to_primitive: Box::new(to_primitive),
            truthy,
            numeric,
            equal: Some(Box::new(move |a, b| {
                match (a.comparison_payload::<T>(), b.comparison_payload::<T>()) {
                    (Some(a), Some(b)) => equal(a, b),
                    _ => false,
                }
            })),
            property_getter: Box::new(move |payload, key| {
                get(
                    payload
                        .downcast_ref::<T>()
                        .expect("property getter payload type"),
                    key,
                )
            }),
        })
    }

    /// Pair a native adapter, such as CustomLogger, with its original host value.
    /// Either type remains available through payload; equality uses the backing
    /// value so repeated wrapping does not create spurious option-change events.
    pub fn with_backing_value<T: Any, U: Any>(
        payload: T,
        backing: U,
        to_primitive: impl Fn(CoercionHint) -> Result<OptionValue, OptionError> + 'static,
        get: impl Fn(&T, &str) -> Result<OptionValue, OptionError> + 'static,
        truthy: bool,
        equal: impl Fn(&U, &U) -> bool + 'static,
    ) -> Rc<Self> {
        Rc::new(Self {
            payload: Box::new(payload),
            backing: Some(Box::new(backing)),
            numeric: None,
            to_primitive: Box::new(to_primitive),
            truthy,
            equal: Some(Box::new(move |a, b| {
                match (a.comparison_payload::<U>(), b.comparison_payload::<U>()) {
                    (Some(a), Some(b)) => equal(a, b),
                    _ => false,
                }
            })),
            property_getter: Box::new(move |payload, key| {
                get(
                    payload
                        .downcast_ref::<T>()
                        .expect("property getter payload type"),
                    key,
                )
            }),
        })
    }

    /// The newly constructed Rc has not escaped and no callback has run, so
    /// adding operator policy cannot fail due to aliases or weak references.
    pub fn with_numeric_backing_value<T: Any, U: Any>(
        payload: T,
        backing: U,
        to_primitive: impl Fn(CoercionHint) -> Result<OptionValue, OptionError> + 'static,
        get: impl Fn(&T, &str) -> Result<OptionValue, OptionError> + 'static,
        truthy: bool,
        equal: impl Fn(&U, &U) -> bool + 'static,
        numeric: Rc<dyn NumericSemantics>,
    ) -> Rc<Self> {
        let mut object =
            Self::with_backing_value(payload, backing, to_primitive, get, truthy, equal);
        Rc::get_mut(&mut object)
            .expect("fresh adapter object has one owner and no weak references")
            .numeric = Some(numeric);
        object
    }

    pub(super) fn has_custom_equality(&self) -> bool {
        self.equal.is_some()
    }
    fn comparison_payload<T: Any>(&self) -> Option<&T> {
        match &self.backing {
            Some(backing) => backing.downcast_ref(),
            None => self.payload.downcast_ref(),
        }
    }
    pub fn backing<T: Any>(&self) -> Option<&T> {
        self.backing.as_ref().and_then(|value| value.downcast_ref())
    }

    pub(super) fn add_left(&self, rhs: &OptionValue) -> Option<Result<OptionValue, OptionError>> {
        self.numeric
            .as_ref()
            .and_then(|numeric| numeric.add_left(rhs))
    }
    pub(super) fn add_right(&self, lhs: &OptionValue) -> Option<Result<OptionValue, OptionError>> {
        self.numeric
            .as_ref()
            .and_then(|numeric| numeric.add_right(lhs))
    }

    pub(super) fn concat_utf16(&self) -> Result<Vec<u16>, OptionError> {
        if let Some(result) = self
            .numeric
            .as_ref()
            .and_then(|numeric| numeric.concat_utf16())
        {
            return result;
        }
        self.primitive(CoercionHint::Number)?.text_utf16()
    }

    pub(super) fn less_than(&self, rhs: f64) -> Result<bool, OptionError> {
        match &self.numeric {
            Some(numeric) => numeric.less_than(rhs),
            None => Ok(self.primitive(CoercionHint::Number)?.number()? < rhs),
        }
    }
    pub(super) fn less_equal(&self, rhs: f64) -> Result<bool, OptionError> {
        match &self.numeric {
            Some(numeric) => numeric.less_equal(rhs),
            None => Ok(self.primitive(CoercionHint::Number)?.number()? <= rhs),
        }
    }
    pub(super) fn multiply(&self, rhs: f64) -> Result<f64, OptionError> {
        match &self.numeric {
            Some(numeric) => numeric.multiply(rhs),
            None => Ok(self.primitive(CoercionHint::Number)?.number()? * rhs),
        }
    }

    pub(super) fn truthy(&self) -> bool {
        self.truthy
    }
    pub(super) fn equals(&self, other: &Self) -> Option<bool> {
        self.equal.as_ref().map(|equal| equal(self, other))
    }

    pub fn get_property(&self, key: &str) -> Result<OptionValue, OptionError> {
        (self.property_getter)(self.payload.as_ref(), key)
    }

    pub fn plain<T: Any>(payload: T) -> Rc<Self> {
        Self::new(payload, |_| Ok(OptionValue::from("[object Object]")))
    }

    pub(super) fn primitive(&self, hint: CoercionHint) -> Result<OptionValue, OptionError> {
        let primitive = (self.to_primitive)(hint)?;
        if matches!(primitive, OptionValue::Object(_)) {
            Err(CoercionError::new("Cannot convert object to primitive value").into())
        } else {
            Ok(primitive)
        }
    }

    pub fn payload<T: Any>(&self) -> Option<&T> {
        self.payload.downcast_ref().or_else(|| {
            self.backing
                .as_ref()
                .and_then(|backing| backing.downcast_ref())
        })
    }
}

impl fmt::Debug for OptionObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OptionObject").finish_non_exhaustive()
    }
}
