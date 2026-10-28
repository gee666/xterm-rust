//! Scroll state and animation policy from the upstream browser scroll system.
//! DOM scrollbars and viewport ownership are separate from these algorithms.

/// Known source constants plus unvalidated JavaScript option values. Unknown
/// values follow the source's automatic-visibility branch without coercion.
#[derive(Clone, Debug)]
pub enum ScrollbarVisibility {
    Auto,
    Hidden,
    Visible,
    Other(xterm_core::options::OptionValue),
}
impl ScrollbarVisibility {
    pub fn from_value(value: xterm_core::options::OptionValue) -> Self {
        use xterm_core::options::OptionValue as V;
        if value == V::Number(1.0) {
            Self::Auto
        } else if value == V::Number(2.0) {
            Self::Hidden
        } else if value == V::Number(3.0) {
            Self::Visible
        } else {
            Self::Other(value)
        }
    }
    pub fn value(&self) -> xterm_core::options::OptionValue {
        self.clone().into()
    }
    pub fn strict_eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
    pub fn is_hidden(&self) -> bool {
        self.value() == xterm_core::options::OptionValue::Number(2.0)
    }
    pub fn is_visible(&self) -> bool {
        self.value() == xterm_core::options::OptionValue::Number(3.0)
    }
}
impl PartialEq for ScrollbarVisibility {
    fn eq(&self, other: &Self) -> bool {
        self.strict_eq(other)
    }
}
impl From<ScrollbarVisibility> for xterm_core::options::OptionValue {
    fn from(value: ScrollbarVisibility) -> Self {
        match value {
            ScrollbarVisibility::Auto => Self::Number(1.0),
            ScrollbarVisibility::Hidden => Self::Number(2.0),
            ScrollbarVisibility::Visible => Self::Number(3.0),
            ScrollbarVisibility::Other(value) => value,
        }
    }
}
pub mod animation;
mod math;
pub mod scrollable;
pub mod scrollbar_state;
pub mod state;
pub mod visibility;
pub mod wheel_classifier;
