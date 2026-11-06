// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
use super::*;
use xterm_core::{decoration::nullish, options::OptionValue};
fn property(decoration: &Decoration, key: &str) -> Result<OptionValue, JsValue> {
    decoration
        .property(key)
        .map_err(|e| js_error::to_js_error(&e))
}
fn number(value: &OptionValue) -> Result<f64, JsValue> {
    value.to_number().map_err(|e| js_error::to_js_error(&e))
}
fn or_one(value: OptionValue) -> OptionValue {
    if value.is_truthy() {
        value
    } else {
        1.0.into()
    }
}
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
        let layer = property(decoration, "layer")?;
        element
            .class_list()
            .toggle_with_force("xterm-decoration-top-layer", layer == "top".into())?;
        self.update_size(decoration, &element, None)?;
        let x = nullish(property(decoration, "x")?, 0.0.into());
        if x.is_truthy() && number(&x)? > self.viewport()?.2 as f64 {
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
            self.update_size(decoration, &entry.element, Some(line))?;
        }
        decoration
            .fire_render(entry.handle)
            .map_err(|e| js_error::to_js_error(&e))
    }
    fn update_size(
        &self,
        decoration: &Decoration,
        element: &HtmlElement,
        line: Option<f64>,
    ) -> Result<(), JsValue> {
        let style = element.style();
        let width = or_one(property(decoration, "width")?);
        let cell_width = (self.host.dimensions)()?.css.cell.width;
        style.set_property("width", &px(round(number(&width)? * cell_width)))?;
        let height = or_one(property(decoration, "height")?);
        let cell_height = (self.host.dimensions)()?.css.cell.height;
        style.set_property("height", &px(number(&height)? * cell_height))?;
        let line = match line {
            Some(line) => line,
            None => decoration.marker().line_number() - self.viewport()?.0,
        };
        let cell_height = (self.host.dimensions)()?.css.cell.height;
        style.set_property("top", &px(line * cell_height))?;
        style.set_property(
            "line-height",
            &px((self.host.dimensions)()?.css.cell.height),
        )
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
        let x = nullish(property(decoration, "x")?, 0.0.into());
        let anchor = property(decoration, "anchor")?;
        let side = if anchor == "right".into() {
            "right"
        } else {
            "left"
        };
        let value = if x.is_truthy() {
            let width = (self.host.dimensions)()?.css.cell.width;
            px(number(&x)? * width)
        } else {
            String::new()
        };
        element.style().set_property(side, &value)
    }
}
fn html(handle: &Rc<dyn std::any::Any>) -> Result<HtmlElement, JsValue> {
    if let Some(element) = handle.downcast_ref::<HtmlElement>() {
        return Ok(element.clone());
    }
    if let Some(value) = handle.downcast_ref::<JsValue>() {
        return value.clone().dyn_into();
    }
    Err(js_sys::TypeError::new("Decoration element is not an HTMLElement").into())
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
