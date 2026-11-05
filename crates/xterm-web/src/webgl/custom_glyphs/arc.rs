// Copyright (c) 2021 The xterm.js authors. All rights reserved. MIT license.
//! SVG endpoint-to-center conversion, preserving the supplied source arithmetic.
use super::types::{DrawCommand as C, GlyphCanvas};
use std::f64::consts::TAU;

pub(super) fn svg_arc<T: GlyphCanvas>(
    canvas: &mut T,
    start: [f64; 2],
    end: [f64; 2],
    radii: [f64; 2],
    phi: f64,
    large: f64,
    sweep: f64,
) -> Result<(), T::Error> {
    if radii[0] == 0.0 || radii[1] == 0.0 {
        return canvas.draw(C::Line(end));
    }
    let [mut rx, mut ry] = [radii[0].abs(), radii[1].abs()];
    let cos = phi.cos();
    let sin = phi.sin();
    let dx = (start[0] - end[0]) / 2.0;
    let dy = (start[1] - end[1]) / 2.0;
    let xp = cos * dx + sin * dy;
    let yp = -sin * dx + cos * dy;
    let mut rx_sq = rx * rx;
    let mut ry_sq = ry * ry;
    let xp_sq = xp * xp;
    let yp_sq = yp * yp;
    let lambda = xp_sq / rx_sq + yp_sq / ry_sq;
    if lambda > 1.0 {
        let scale = lambda.sqrt();
        rx *= scale;
        ry *= scale;
        rx_sq = rx * rx;
        ry_sq = ry * ry;
    }
    let mut sq = (rx_sq * ry_sq - rx_sq * yp_sq - ry_sq * xp_sq) / (rx_sq * yp_sq + ry_sq * xp_sq);
    if sq < 0.0 {
        sq = 0.0;
    }
    let coefficient = (if large == sweep { -1.0 } else { 1.0 }) * sq.sqrt();
    let cxp = coefficient * (rx * yp / ry);
    let cyp = coefficient * -(ry * xp / rx);
    let cx = cos * cxp - sin * cyp + (start[0] + end[0]) / 2.0;
    let cy = sin * cxp + cos * cyp + (start[1] + end[1]) / 2.0;
    let ux = (xp - cxp) / rx;
    let uy = (yp - cyp) / ry;
    let vx = (-xp - cxp) / rx;
    let vy = (-yp - cyp) / ry;
    let start_angle = uy.atan2(ux);
    let mut delta = vy.atan2(vx) - start_angle;
    if sweep == 0.0 && delta > 0.0 {
        delta -= TAU;
    } else if sweep == 1.0 && delta < 0.0 {
        delta += TAU;
    }
    canvas.draw(C::Ellipse(
        [cx, cy, rx, ry, phi, start_angle, start_angle + delta],
        sweep == 0.0,
    ))
}
