use super::*;
use crate::renderer::color_zone_store::{ColorZone, ColorZoneOptions, ColorZonePadding};
use xterm_core::decoration::OverviewPosition;
const BORDER: f64 = 1.0;
#[derive(Default)]
struct Draw {
    width: [f64; 4],
    height: [f64; 4],
    x: [f64; 4],
}
// Upstream keeps these module-global, including cross-ruler reuse when a redraw
// does not refresh dimensions. Preserve that behavior rather than silently fix it.
thread_local! {static DRAW:RefCell<Draw>=RefCell::new(Draw::default());}
fn index(position: Option<OverviewPosition>) -> usize {
    match position.unwrap_or(OverviewPosition::Full) {
        OverviewPosition::Full => 0,
        OverviewPosition::Left => 1,
        OverviewPosition::Center => 2,
        OverviewPosition::Right => 3,
    }
}
impl OverviewRulerRenderer {
    pub(super) fn refresh_canvas_dimensions(&self) -> Result<(), JsValue> {
        if self.disposed.get() || !self.services.render.has_renderer() {
            return Ok(());
        }
        let dimensions = self.services.render.dimensions()?;
        let mut width = self
            .width()?
            .to_utf16()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        width.extend("px".encode_utf16());
        self.style_units("width", &width)?;
        let physical_width = round(number(&self.width()?)? * self.services.browser.dpr()?);
        self.canvas.set_width(uint32(physical_width));
        self.canvas.style().set_property(
            "height",
            &format!(
                "{}px",
                ryu_js::Buffer::new().format(dimensions.css.canvas.height)
            ),
        )?;
        self.canvas
            .set_height(uint32(dimensions.device.canvas.height));
        let width = f64::from(self.canvas.width());
        let outer = ((width - BORDER) / 3.0).floor();
        let inner = ((width - BORDER) / 3.0).ceil();
        DRAW.with(|draw| draw.borrow_mut().width = [width, outer, inner, outer]);
        self.refresh_heights()?;
        DRAW.with(|draw| {
            draw.borrow_mut().x = [BORDER, BORDER, BORDER + outer, BORDER + outer + inner]
        });
        self.refresh_padding()
    }
    pub(super) fn refresh_heights(&self) -> Result<(), JsValue> {
        let full = round(2.0 * self.services.browser.dpr()?);
        DRAW.with(|draw| draw.borrow_mut().height[0] = full);
        let pixels = f64::from(self.canvas.height()) / self.active_length()? as f64;
        let clamped = pixels.clamp(6.0, 12.0);
        let height = round(clamped * self.services.browser.dpr()?);
        DRAW.with(|draw| {
            let mut draw = draw.borrow_mut();
            draw.height[1..].fill(height);
        });
        Ok(())
    }
    pub(super) fn refresh_padding(&self) -> Result<(), JsValue> {
        let height = DRAW.with(|draw| draw.borrow().height);
        let scale = self.active_length()? as f64 / (f64::from(self.canvas.height()) - 1.0);
        let values = height.map(|height| (scale * height).floor());
        self.zones
            .borrow_mut()
            .set_padding(Rc::new(RefCell::new(ColorZonePadding {
                full: Some(values[0]),
                left: Some(values[1]),
                center: Some(values[2]),
                right: Some(values[3]),
            })));
        self.last_length.set(self.normal_length()?);
        Ok(())
    }
    pub(super) fn refresh_decorations(&self) -> Result<(), JsValue> {
        if self.disposed.get() || !self.services.render.has_renderer() {
            return Ok(());
        }
        if self.update_dimensions.get() {
            self.refresh_canvas_dimensions()?;
        }
        self.ctx()?.clear_rect(
            0.0,
            0.0,
            self.canvas.width().into(),
            self.canvas.height().into(),
        );
        self.zones
            .borrow_mut()
            .clear()
            .map_err(|e| js_sys::Error::new(&e.to_string()))?;
        let decorations = self
            .services
            .decorations
            .decorations()
            .map_err(|e| js_sys::Error::new(&e.to_string()))?;
        for decoration in decorations {
            let options = ColorZoneOptions::from_decoration(&decoration)
                .map_err(|e| crate::js_error::to_js_error(&e))?;
            self.zones
                .try_borrow_mut()
                .map_err(|e| crate::js_error::to_js_error(&e))?
                .add(decoration.marker().line_number(), options.as_ref())
                .map_err(|e| crate::js_error::to_js_error(&e))?;
        }
        self.ctx()?.set_line_width(1.0);
        self.render_outline()?;
        let zones = self
            .zones
            .borrow_mut()
            .zones()
            .map_err(|e| js_sys::Error::new(&e.to_string()))?;
        let handles = zones.borrow().clone();
        for zone in &handles {
            if zone.borrow().position != Some(OverviewPosition::Full) {
                let zone = zone.borrow().clone();
                self.render_zone(&zone)?;
            }
        }
        for zone in &handles {
            if zone.borrow().position == Some(OverviewPosition::Full) {
                let zone = zone.borrow().clone();
                self.render_zone(&zone)?;
            }
        }
        self.update_dimensions.set(false);
        Ok(())
    }
    fn render_outline(&self) -> Result<(), JsValue> {
        let colors = self.services.theme.colors();
        let border = colors.borrow().overview_ruler_border.clone();
        self.fill_style(&border.css_units())?;
        self.ctx()?
            .fill_rect(0.0, 0.0, BORDER, self.canvas.height().into());
        let overview = optional(&self.scrollbar()?, "overviewRuler")?;
        if optional(&overview, "showTopBorder")?.is_truthy() {
            self.ctx()?
                .fill_rect(BORDER, 0.0, f64::from(self.canvas.width()) - BORDER, BORDER);
        }
        let overview = optional(&self.scrollbar()?, "overviewRuler")?;
        if optional(&overview, "showBottomBorder")?.is_truthy() {
            self.ctx()?.fill_rect(
                BORDER,
                f64::from(self.canvas.height()) - BORDER,
                f64::from(self.canvas.width()) - BORDER,
                self.canvas.height().into(),
            );
        }
        Ok(())
    }
    fn render_zone(&self, zone: &ColorZone) -> Result<(), JsValue> {
        self.fill_style(&zone.color)?;
        let lane = index(zone.position);
        let (x, width, height) = DRAW.with(|draw| {
            let draw = draw.borrow();
            (draw.x[lane], draw.width[lane], draw.height[lane])
        });
        let y = round(
            (f64::from(self.canvas.height()) - 1.0)
                * (zone.start_buffer_line / self.active_length()? as f64)
                - height / 2.0,
        );
        let height = round(
            (f64::from(self.canvas.height()) - 1.0)
                * ((zone.end_buffer_line - zone.start_buffer_line) / self.active_length()? as f64)
                + height,
        );
        self.ctx()?.fill_rect(x, y, width, height);
        Ok(())
    }
    fn fill_style(&self, units: &[u16]) -> Result<(), JsValue> {
        if !js_sys::Reflect::set(
            self.ctx()?.as_ref(),
            &"fillStyle".into(),
            crate::utf16::to_js_string(units).as_ref(),
        )? {
            return Err(js_sys::TypeError::new("canvas fillStyle is not writable").into());
        }
        Ok(())
    }
    fn style_units(&self, name: &str, units: &[u16]) -> Result<(), JsValue> {
        let style = self.canvas.style();
        let method: js_sys::Function =
            js_sys::Reflect::get(style.as_ref(), &"setProperty".into())?.unchecked_into();
        method.call2(
            style.as_ref(),
            &name.into(),
            crate::utf16::to_js_string(units).as_ref(),
        )?;
        Ok(())
    }
}
