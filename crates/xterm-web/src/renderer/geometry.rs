// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use crate::api::{CharacterDimensions, CssDimensions, DeviceDimensions, RenderDimensions, Size};
fn round(value: f64) -> f64 {
    let floor = value.floor();
    if value - floor < 0.5 {
        floor
    } else {
        value.ceil()
    }
}
/// Source _updateDimensions rounding order, including fractional DPR/spacing.
pub fn dimensions(
    char_size: Size,
    dpr: f64,
    cols: usize,
    rows: usize,
    letter_spacing: f64,
    line_height: f64,
) -> RenderDimensions {
    let character = CharacterDimensions {
        width: char_size.width * dpr,
        height: (char_size.height * dpr).ceil(),
        left: 0.0,
        top: 0.0,
    };
    let cell = Size {
        width: character.width + round(letter_spacing),
        height: (character.height * line_height).floor(),
    };
    let canvas = Size {
        width: cell.width * cols as f64,
        height: cell.height * rows as f64,
    };
    let css_canvas = Size {
        width: round(canvas.width / dpr),
        height: round(canvas.height / dpr),
    };
    RenderDimensions {
        device: DeviceDimensions {
            character,
            cell,
            canvas,
        },
        css: CssDimensions {
            canvas: css_canvas,
            cell: Size {
                width: css_canvas.width / cols as f64,
                height: css_canvas.height / rows as f64,
            },
        },
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fractional_dpr_preserves_source_rounding_order() {
        let result = dimensions(
            Size {
                width: 7.25,
                height: 13.1,
            },
            1.25,
            80,
            24,
            0.5,
            1.2,
        );
        assert_eq!(result.device.character.height, 17.0);
        assert_eq!(result.device.cell.width, 10.0625);
        assert_eq!(result.device.cell.height, 20.0);
        assert_eq!(
            result.css.canvas,
            Size {
                width: 644.0,
                height: 384.0
            }
        );
        assert_eq!(
            result.css.cell,
            Size {
                width: 8.05,
                height: 16.0
            }
        );
    }
    #[test]
    fn zero_metrics_and_negative_half_spacing_are_not_saturated() {
        let result = dimensions(Size::default(), 2.0, 80, 24, -0.5, 1.0);
        assert_eq!(result.css.canvas, Size::default());
        let result = dimensions(
            Size {
                width: 10.0,
                height: 20.0,
            },
            2.0,
            80,
            24,
            -1.5,
            1.0,
        );
        assert_eq!(result.device.cell.width, 19.0);
    }
}
