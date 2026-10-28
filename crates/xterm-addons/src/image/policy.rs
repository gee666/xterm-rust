// Copyright (c) 2020 The xterm.js authors. MIT license.
//! Source image-control state and ordered effects, separate from parser/DOM wiring.
use super::numeric::fixed_zero;
use super::options::ImageOptions;
use super::storage::{CellSize, CELL_SIZE_DEFAULT};

const MAX_SIXEL_PALETTE: f64 = 4096.0;
const DEC_SIXEL_DISPLAY: f64 = 80.0;
const GRAPHICS_COLORS: f64 = 1.0;
const GRAPHICS_SIXEL_GEOMETRY: f64 = 2.0;
const GRAPHICS_READ: f64 = 1.0;
const GRAPHICS_SET_DEFAULT: f64 = 2.0;
const GRAPHICS_SET: f64 = 3.0;
const GRAPHICS_READ_MAX: f64 = 4.0;
const FALLBACK_COLUMNS: f64 = 80.0;
const FALLBACK_ROWS: f64 = 24.0;

#[derive(Clone, Copy)]
#[repr(u8)]
enum GraphicsStatus {
    Success = 0,
    ItemError = 1,
    ActionError = 2,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ImageParameter {
    Number(f64),
    Subparams(Vec<f64>),
    Undefined,
}
impl ImageParameter {
    fn number(&self) -> Option<f64> {
        if let Self::Number(value) = self {
            Some(*value)
        } else {
            None
        }
    }
    fn truthy(&self) -> bool {
        match self {
            Self::Number(value) => truthy(*value),
            Self::Subparams(_) => true,
            Self::Undefined => false,
        }
    }
    fn text(&self) -> String {
        match self {
            Self::Number(value) => number_text(*value),
            Self::Subparams(values) => values
                .iter()
                .map(|v| number_text(*v))
                .collect::<Vec<_>>()
                .join(","),
            Self::Undefined => "undefined".to_owned(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageControlEffect {
    Reply(Vec<u16>),
    ClearStorage,
    ResetHandlers,
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use = "control effects must execute in order against the actual terminal and handlers"]
pub struct ImageControlAction {
    pub handled: bool,
    pub effects: Vec<ImageControlEffect>,
}
impl ImageControlAction {
    fn empty(handled: bool) -> Self {
        Self {
            handled,
            effects: Vec::new(),
        }
    }
    fn reply(text: String) -> Self {
        Self {
            handled: true,
            effects: vec![ImageControlEffect::Reply(text.encode_utf16().collect())],
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphicsGeometry {
    pub canvas: Option<CellSize>,
    pub columns: f64,
    pub rows: f64,
}
impl Default for GraphicsGeometry {
    fn default() -> Self {
        Self {
            canvas: None,
            columns: FALLBACK_COLUMNS,
            rows: FALLBACK_ROWS,
        }
    }
}

pub struct ImageControlPolicy {
    pub options: ImageOptions,
    defaults: ImageOptions,
}
impl ImageControlPolicy {
    pub fn new(options: ImageOptions) -> Self {
        Self {
            options,
            defaults: options,
        }
    }
    pub fn constructor_defaults(&self) -> ImageOptions {
        self.defaults
    }
    pub fn reset(&mut self) -> ImageControlAction {
        self.options.sixel_scrolling = self.defaults.sixel_scrolling;
        self.options.sixel_palette_limit = self.defaults.sixel_palette_limit;
        ImageControlAction {
            handled: false,
            effects: vec![
                ImageControlEffect::ClearStorage,
                ImageControlEffect::ResetHandlers,
            ],
        }
    }
    pub fn decset(&mut self, parameters: &[ImageParameter]) -> ImageControlAction {
        self.set_scrolling(parameters, false)
    }
    pub fn decrst(&mut self, parameters: &[ImageParameter]) -> ImageControlAction {
        self.set_scrolling(parameters, true)
    }
    fn set_scrolling(
        &mut self,
        parameters: &[ImageParameter],
        scrolling: bool,
    ) -> ImageControlAction {
        for parameter in parameters {
            if parameter.number() == Some(DEC_SIXEL_DISPLAY) {
                self.options.sixel_scrolling = scrolling;
            }
        }
        ImageControlAction::empty(false)
    }
    pub fn primary_device_attributes(&self, parameters: &[ImageParameter]) -> ImageControlAction {
        if parameters.first().is_some_and(ImageParameter::truthy) {
            return ImageControlAction::empty(true);
        }
        if self.options.sixel_support {
            ImageControlAction::reply("\x1b[?62;4;9;22c".to_owned())
        } else {
            ImageControlAction::empty(false)
        }
    }
    pub fn graphics_attributes(
        &mut self,
        parameters: &[ImageParameter],
        geometry: GraphicsGeometry,
    ) -> ImageControlAction {
        if parameters.len() < 2 {
            return ImageControlAction::empty(true);
        }
        let item = &parameters[0];
        if item.number() == Some(GRAPHICS_COLORS) {
            return match parameters[1].number() {
                Some(GRAPHICS_READ) => report(
                    item,
                    GraphicsStatus::Success,
                    Some(number_text(self.options.sixel_palette_limit)),
                ),
                Some(GRAPHICS_SET_DEFAULT) => {
                    self.options.sixel_palette_limit = self.defaults.sixel_palette_limit;
                    let mut action = report(
                        item,
                        GraphicsStatus::Success,
                        Some(number_text(self.options.sixel_palette_limit)),
                    );
                    // The reply can reenter the addon. The executor must read live
                    // options and handler membership only after that callback returns.
                    action.effects.push(ImageControlEffect::ResetHandlers);
                    action
                }
                Some(GRAPHICS_SET) => {
                    match parameters
                        .get(2)
                        .and_then(ImageParameter::number)
                        .filter(|v| *v <= MAX_SIXEL_PALETTE)
                    {
                        Some(value) => {
                            self.options.sixel_palette_limit = value;
                            report(item, GraphicsStatus::Success, Some(number_text(value)))
                        }
                        None => report(item, GraphicsStatus::ActionError, None),
                    }
                }
                Some(GRAPHICS_READ_MAX) => report(
                    item,
                    GraphicsStatus::Success,
                    Some(number_text(MAX_SIXEL_PALETTE)),
                ),
                _ => report(item, GraphicsStatus::ActionError, None),
            };
        }
        if item.number() == Some(GRAPHICS_SIXEL_GEOMETRY) {
            return match parameters[1].number() {
                Some(GRAPHICS_READ) => {
                    let (mut width, mut height) = geometry
                        .canvas
                        .map_or((f64::NAN, f64::NAN), |cell| (cell.width, cell.height));
                    if !truthy(width) || !truthy(height) {
                        width = or(geometry.columns, FALLBACK_COLUMNS) * CELL_SIZE_DEFAULT.width;
                        height = or(geometry.rows, FALLBACK_ROWS) * CELL_SIZE_DEFAULT.height;
                    }
                    let values = if width * height < self.options.pixel_limit {
                        format!("{};{}", fixed_zero(width), fixed_zero(height))
                    } else {
                        square(self.options.pixel_limit)
                    };
                    report(item, GraphicsStatus::Success, Some(values))
                }
                Some(GRAPHICS_READ_MAX) => report(
                    item,
                    GraphicsStatus::Success,
                    Some(square(self.options.pixel_limit)),
                ),
                _ => report(item, GraphicsStatus::ActionError, None),
            };
        }
        report(item, GraphicsStatus::ItemError, None)
    }
}
fn truthy(value: f64) -> bool {
    value != 0.0 && !value.is_nan()
}
fn or(value: f64, fallback: f64) -> f64 {
    if truthy(value) {
        value
    } else {
        fallback
    }
}
fn number_text(value: f64) -> String {
    ryu_js::Buffer::new().format(value).to_owned()
}
fn square(limit: f64) -> String {
    let value = number_text(limit.sqrt().floor());
    format!("{value};{value}")
}
fn report(
    item: &ImageParameter,
    status: GraphicsStatus,
    values: Option<String>,
) -> ImageControlAction {
    let suffix = values.map_or(String::new(), |values| format!(";{values}"));
    ImageControlAction::reply(format!(
        "\x1b[?{};{status}{suffix}S",
        item.text(),
        status = status as u8
    ))
}
