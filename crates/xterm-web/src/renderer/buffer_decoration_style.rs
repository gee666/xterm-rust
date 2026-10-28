// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
use super::*;
use xterm_core::decoration::{DecorationAnchor, DecorationLayer};
impl BufferDecorationRenderer {
    fn viewport(&self) -> Result<(f64, usize, usize), JsValue> {
        let active = self.host.buffers.active();
        let buffer = active.try_borrow().map_err(|e| js_error::to_js_error(&e))?;
        let dimensions = self.host.buffers.dimensions();
        Ok((
            buffer.ydisp as f64,
            dimensions.rows.get(),
            dimensions.cols.get(),
        ))
    }
    fn create_element(&self, decoration: &Decoration) -> Result<HtmlElement, JsValue> {
        let element: HtmlElement = self.host.document.create_element("div")?.dyn_into()?;
        element.class_list().add_1("xterm-decoration")?;
        let layer = decoration.options().borrow().layer;
        element.class_list().toggle_with_force(
            "xterm-decoration-top-layer",
            layer == Some(DecorationLayer::Top),
        )?;
        let (ydisp, _, cols) = self.viewport()?;
        let line = decoration.marker().line_number() - ydisp;
        self.update_size(decoration, &element, line)?;
        let x = decoration.options().borrow().x.unwrap_or(0.0);
        if truthy(x) && x > cols as f64 {
            element.style().set_property("display", "none")?;
        }
        self.refresh_x(decoration, Some(&element))?;
        Ok(element)
    }
    pub(super) fn refresh_style(self: &Rc<Self>, decoration: &Decoration) -> Result<(), JsValue> {
        let (ydisp, rows, _) = self.viewport()?;
        let line = decoration.marker().line_number() - ydisp;
        if line < 0.0 || line >= rows as f64 {
            if let Some(handle) = decoration.element() {
                html(&handle)?.style().set_property("display", "none")?;
                decoration
                    .fire_render(handle)
                    .map_err(|e| js_error::to_js_error(&e))?;
            }
            return Ok(());
        }
        let existing = self
            .entries
            .borrow()
            .iter()
            .find(|entry| entry.decoration.ptr_eq(decoration))
            .cloned();
        let entry = match existing {
            Some(entry) => entry,
            None => {
                let element = self.create_element(decoration)?;
                let handle: Rc<dyn std::any::Any> = Rc::new(element.clone());
                decoration.set_element(Some(Rc::clone(&handle)));
                let entry = Entry {
                    decoration: decoration.clone(),
                    element: element.clone(),
                    handle,
                };
                self.entries.borrow_mut().push(entry.clone());
                self.container.append_child(&element)?;
                let weak = Rc::downgrade(self);
                decoration.on_dispose().subscribe(move |_| {
                    if let Some(renderer) = weak.upgrade() {
                        renderer
                            .entries
                            .borrow_mut()
                            .retain(|entry| !entry.element.is_same_node(Some(&element)));
                    }
                    element.remove();
                });
                entry
            }
        };
        let alternate = self.alt_active.get();
        entry
            .element
            .style()
            .set_property("display", if alternate { "none" } else { "block" })?;
        if !alternate {
            self.update_size(decoration, &entry.element, line)?;
        }
        decoration
            .fire_render(entry.handle)
            .map_err(|e| js_error::to_js_error(&e))
    }
    fn update_size(
        &self,
        decoration: &Decoration,
        element: &HtmlElement,
        line: f64,
    ) -> Result<(), JsValue> {
        let (width, height) = {
            let options = decoration.options();
            let options = options.borrow();
            (
                options.width.filter(|value| truthy(*value)).unwrap_or(1.0),
                options.height.filter(|value| *value != 0).unwrap_or(1),
            )
        };
        let dimensions = (self.host.dimensions)()?;
        let cell = dimensions.css.cell;
        let style = element.style();
        style.set_property("width", &px(round(width * cell.width)))?;
        style.set_property("height", &px(f64::from(height) * cell.height))?;
        style.set_property("top", &px(line * cell.height))?;
        style.set_property("line-height", &px(cell.height))
    }
    pub(super) fn refresh_x(
        &self,
        decoration: &Decoration,
        element: Option<&HtmlElement>,
    ) -> Result<(), JsValue> {
        let element = match element {
            Some(element) => element.clone(),
            None => match decoration.element() {
                Some(handle) => html(&handle)?,
                None => return Ok(()),
            },
        };
        let (x, anchor) = {
            let options = decoration.options();
            let options = options.borrow();
            (
                options.x.unwrap_or(0.0),
                options.anchor.unwrap_or(DecorationAnchor::Left),
            )
        };
        let value = if truthy(x) {
            px(x * (self.host.dimensions)()?.css.cell.width)
        } else {
            String::new()
        };
        element.style().set_property(
            if anchor == DecorationAnchor::Right {
                "right"
            } else {
                "left"
            },
            &value,
        )
    }
}
fn html(handle: &Rc<dyn std::any::Any>) -> Result<HtmlElement, JsValue> {
    handle
        .downcast_ref::<HtmlElement>()
        .cloned()
        .ok_or_else(|| js_sys::TypeError::new("Decoration element is not an HTMLElement").into())
}
fn truthy(value: f64) -> bool {
    value != 0.0 && !value.is_nan()
}
fn round(value: f64) -> f64 {
    let floor = value.floor();
    if value - floor < 0.5 {
        floor
    } else {
        value.ceil()
    }
}
fn px(value: f64) -> String {
    format!("{}px", ryu_js::Buffer::new().format(value))
}
