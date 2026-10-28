// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
use super::*;
use crate::scroll::fast_dom_node::{assign_style, begin_style};
use crate::terminal::options::into_js;

impl SmoothScrollableElement {
    pub(super) fn build_dom(&self, element: HtmlElement) -> Result<(), JsValue> {
        let document = self
            .environment
            .main_window
            .document()
            .ok_or_else(|| js_sys::Error::new("scrollable requires a document"))?;
        let node: HtmlElement = document.create_element("div")?.unchecked_into();
        let class_name = self.options.borrow().class_name.clone();
        let class_name = V::from("xterm-scrollable-element ")
            .js_add(&class_name)
            .map_err(|error| crate::js_error::to_js_error(&error))?;
        interop::set(node.as_ref(), "className", &into_js(&class_name)?)?;
        node.set_attribute("role", "presentation")?;
        assign_style(&begin_style(&node)?, "position", &"relative".into())?;
        node.append_child(&element)?;
        node.append_child(&self.horizontal()?.dom_node())?;
        node.append_child(&self.vertical()?.dom_node())?;
        *self.node.borrow_mut() = Some(node.clone());
        if self.options.borrow().use_shadows.is_truthy() {
            let mut shadows = Vec::new();
            for _ in 0..3 {
                let shadow = FastDomNode::new(document.create_element("div")?.unchecked_into());
                shadow.set_class_name("xterm-shadow")?;
                node.append_child(&shadow.dom_node)?;
                shadows.push(shadow);
            }
            let shadows: [FastDomNode; 3] = shadows
                .try_into()
                .map_err(|_| js_sys::Error::new("three scroll shadows must be created"))?;
            *self.shadows.borrow_mut() = Some(shadows);
        }
        let listen = self.options.borrow().listen_on_dom_node.clone();
        *self.listen_on.borrow_mut() = Some(if interop::nullish(&listen) {
            node.into()
        } else {
            listen.unchecked_into()
        });
        Ok(())
    }
    pub fn update_class_name(&self, value: V) -> Result<(), JsValue> {
        self.options.borrow_mut().class_name = value;
        if self.environment.platform.is_mac {
            let current = self.options.borrow().class_name.clone();
            let next = current
                .js_add(&V::from(" xterm-mac"))
                .map_err(|error| crate::js_error::to_js_error(&error))?;
            self.options.borrow_mut().class_name = next;
        }
        let current = self.options.borrow().class_name.clone();
        let text = V::from("xterm-scrollable-element ")
            .js_add(&current)
            .map_err(|error| crate::js_error::to_js_error(&error))?;
        interop::set(self.get_dom_node()?.as_ref(), "className", &into_js(&text)?)
    }
    pub fn update_options(self: &Rc<Self>, raw: &JsValue) -> Result<(), JsValue> {
        if let Some(value) = read_change(raw, "handleMouseWheel")? {
            self.options.borrow_mut().handle_mouse_wheel = value;
            self.set_listening_to_wheel()?;
        }
        macro_rules! option {
            ($key:literal, $field:ident) => {
                if let Some(value) = read_change(raw, $key)? {
                    self.options.borrow_mut().$field = value;
                }
            };
        }
        option!(
            "mouseWheelScrollSensitivity",
            mouse_wheel_scroll_sensitivity
        );
        option!("fastScrollSensitivity", fast_scroll_sensitivity);
        option!("scrollPredominantAxis", scroll_predominant_axis);
        if let Some(value) = read_change(raw, "horizontal")? {
            self.options.borrow_mut().horizontal =
                super::super::ScrollbarVisibility::from_value(value);
        }
        if let Some(value) = read_change(raw, "vertical")? {
            self.options.borrow_mut().vertical =
                super::super::ScrollbarVisibility::from_value(value);
        }
        option!("horizontalHasArrows", horizontal_has_arrows);
        option!("verticalHasArrows", vertical_has_arrows);
        option!("horizontalScrollbarSize", horizontal_scrollbar_size);
        option!("verticalScrollbarSize", vertical_scrollbar_size);
        option!("scrollByPage", scroll_by_page);
        self.horizontal()?.update_options(&self.options)?;
        self.vertical()?.update_options(&self.options)?;
        if !self.options.borrow().lazy_render.is_truthy() {
            self.render()?;
        }
        Ok(())
    }
    pub(super) fn handle_scroll(self: &Rc<Self>, event: &ScrollEvent) -> Result<(), JsValue> {
        let horizontal = self.horizontal()?.handle_scroll(event)?;
        self.should_render
            .set(horizontal || self.should_render.get());
        let vertical = self.vertical()?.handle_scroll(event)?;
        self.should_render.set(vertical || self.should_render.get());
        if self.options.borrow().use_shadows.is_truthy() {
            self.should_render.set(true);
        }
        self.reveal()?;
        if !self.options.borrow().lazy_render.is_truthy() {
            self.render()?;
        }
        Ok(())
    }
    pub fn render_now(&self) -> Result<(), JsValue> {
        if !self.options.borrow().lazy_render.is_truthy() {
            return Err(
                js_sys::Error::new("Please use `lazyRender` together with `renderNow`!").into(),
            );
        }
        self.render()
    }
    fn render(&self) -> Result<(), JsValue> {
        if !self.should_render.replace(false) {
            return Ok(());
        }
        self.horizontal()?.render()?;
        self.vertical()?.render()?;
        if self.options.borrow().use_shadows.is_truthy() {
            let position = self.model.current_scroll_position();
            let left = if position.left > 0.0 {
                " xterm-shadow-left"
            } else {
                ""
            };
            let top = if position.top > 0.0 {
                " xterm-shadow-top"
            } else {
                ""
            };
            let corner = if position.left > 0.0 || position.top > 0.0 {
                " xterm-shadow-top-left-corner"
            } else {
                ""
            };
            let shadows = self
                .shadows
                .borrow()
                .clone()
                .ok_or_else(|| js_sys::TypeError::new("scroll shadows are not initialized"))?;
            shadows[0].set_class_name(&format!("xterm-shadow{left}"))?;
            shadows[1].set_class_name(&format!("xterm-shadow{top}"))?;
            shadows[2].set_class_name(&format!("xterm-shadow{corner}{top}{left}"))?;
        }
        Ok(())
    }
}
