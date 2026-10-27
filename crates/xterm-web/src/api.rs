//! Shared public validation and browser render geometry.
pub use xterm_core::public_api::validation::{
    verify_integers, verify_mutable_option, verify_positive_integers, verify_proposed_api, ApiError,
};

/// Matches the externally visible subset of IRenderDimensions. Character
/// offsets are device pixels, and canvas/cell dimensions are not rounded here.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CssDimensions {
    pub canvas: Size,
    pub cell: Size,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CharacterDimensions {
    pub width: f64,
    pub height: f64,
    pub left: f64,
    pub top: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DeviceDimensions {
    pub canvas: Size,
    pub cell: Size,
    pub character: CharacterDimensions,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderDimensions {
    pub css: CssDimensions,
    pub device: DeviceDimensions,
}
