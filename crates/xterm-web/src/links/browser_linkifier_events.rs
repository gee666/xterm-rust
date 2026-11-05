// Copyright (c) 2019-2026 The xterm.js authors. All rights reserved. MIT license.
use super::{BrowserLink, CachedBrowserLink, State};
use crate::links::{Position, ProviderRequest};
use js_sys::Set;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, Event, MouseEvent};

impl State {
    pub(super) fn mouse_move(self: &Rc<Self>, event: MouseEvent) -> Result<(), JsValue> {
        *self.last_event.borrow_mut() = Some(event.clone());
        let Some(position) = (self.coordinates)(&event)? else {
            return Ok(());
        };
        self.mouse_out.set(false);
        for target in event.composed_path().iter() {
            let Some(element) = target.dyn_ref::<Element>() else {
                continue;
            };
            if element.class_list().contains("xterm") {
                break;
            }
            if element.class_list().contains("xterm-hover") {
                return Ok(());
            }
        }
        if self.last_cell.get() == Some(position) {
            return Ok(());
        }
        if self.active_line.get() != Some(position.y) || self.resized.get() {
            self.clear_current()?;
            self.ask(position, false)?;
            self.resized.set(false);
        } else {
            let current = self.hover.borrow().current().cloned();
            let contains = match current {
                Some(link) => link.contains_with_columns(position, || self.cols.get())?,
                None => false,
            };
            if !contains {
                self.clear_current()?;
                self.ask(position, true)?;
            }
        }
        self.last_cell.set(Some(position));
        Ok(())
    }
    pub(super) fn mouse_up(&self, event: Event) -> Result<(), JsValue> {
        // Keep the source's no-current-link return before event/geometry reads.
        if self.hover.borrow().current().is_none() {
            return Ok(());
        }
        let event = event
            .dyn_into::<MouseEvent>()
            .map_err(|_| JsValue::from_str("mouseup was not a MouseEvent"))?;
        let Some(position) = (self.coordinates)(&event)? else {
            return Ok(());
        };
        let down = self.hover.borrow().mouse_down_link();
        if let Some(down) = down {
            // A geometry getter can reenter and clear the current link.
            let current = self.hover.borrow().current().cloned().ok_or_else(|| {
                js_sys::TypeError::new("Cannot read properties of undefined (reading 'link')")
            })?;
            if down.equals(&current)? {
                let current = self.hover.borrow().current().cloned().ok_or_else(|| {
                    js_sys::TypeError::new("Cannot read properties of undefined (reading 'link')")
                })?;
                if current.contains_with_columns(position, || self.cols.get())? {
                    let current = self.hover.borrow().current().cloned().ok_or_else(|| {
                        js_sys::TypeError::new(
                            "Cannot read properties of undefined (reading 'link')",
                        )
                    })?;
                    current.invoke("activate", &event)?;
                }
            }
        }
        Ok(())
    }
    pub(super) fn clear_current(&self) -> Result<(), JsValue> {
        let current = self.hover.borrow().current().cloned();
        let event = self.last_event.borrow().clone();
        let (Some(link), Some(event)) = (current, event) else {
            return Ok(());
        };
        let values = link.decoration_values.borrow().clone();
        if let Some(values) = values {
            link.hovered.set(false);
            if values.0.is_truthy() {
                self.fire_underline(&link, false)?;
            }
            if self.current_pointer()? {
                self.element.class_list().remove_1("xterm-cursor-pointer")?;
            }
        }
        link.invoke("leave", &event)?;
        self.hover.borrow_mut().clear_current();
        Ok(())
    }
    pub(super) fn ask(self: &Rc<Self>, position: Position, use_cache: bool) -> Result<(), JsValue> {
        if !use_cache || self.active_line.get().is_none() {
            // Map.forEach retains this map identity but observes later insertions.
            let replies = self.replies.borrow().clone();
            let mut ordinal = 0;
            while let Some(group) = replies.disposal_group(ordinal) {
                if let Some(group) = group {
                    let length = group.borrow().len();
                    for index in 0..length {
                        let link = group.borrow().get(index).cloned();
                        if let Some(link) = link {
                            link.dispose()?;
                        }
                    }
                }
                ordinal += 1;
            }
            *self.replies.borrow_mut() = Rc::new(Default::default());
            self.active_line.set(Some(position.y));
        }
        let request = ProviderRequest::new(position);
        let mut index = 0;
        while let Some(provider) = self.providers.get(index) {
            if use_cache {
                let replies = self.replies.borrow().clone();
                replies.try_cached_selecting(
                    index,
                    &request,
                    self.providers.len(),
                    |link| self.probe(link, position),
                    |link| self.select(link),
                )?;
            } else {
                let weak = Rc::downgrade(self);
                let mouse_out = self.mouse_out.clone();
                let request = request.clone();
                provider.provide_links_lazy(
                    position.y,
                    Box::new(move |resolve| {
                        if mouse_out.get() {
                            return Ok(());
                        }
                        // Disposal clears the last event, not the source's mouse-out
                        // flag. A late raw reply still observes map and its errors.
                        let data = resolve()?.map(|links| {
                            links
                                .into_iter()
                                .map(|link| Rc::new(CachedBrowserLink::new(link)))
                                .collect()
                        });
                        let Some(state) = weak.upgrade().filter(|s| !s.disposed.get()) else {
                            return Ok(());
                        };
                        let replies = state.replies.borrow().clone();
                        replies.try_receive_selecting(
                            index,
                            data,
                            &request,
                            state.providers.len(),
                            |link| state.probe(link, request.position),
                            |link| state.select(link),
                        )?;
                        // Selection callbacks can replace the active reply map.
                        let replies = state.replies.borrow().clone();
                        let occupied = Set::new(&JsValue::UNDEFINED);
                        replies.try_filter_complete(state.providers.len(), |link| {
                            link.keep_nonintersecting(
                                request.position.y,
                                state.cols.get(),
                                &occupied,
                            )
                        })?;
                        Ok(())
                    }),
                )?;
            }
            index += 1;
        }
        Ok(())
    }
    fn probe(
        &self,
        link: Option<&Rc<CachedBrowserLink>>,
        position: Position,
    ) -> Result<bool, JsValue> {
        link.ok_or_else(|| {
            js_sys::TypeError::new("Cannot read properties of undefined (reading 'link')")
        })?
        .contains_with_columns(position, || self.cols.get())
    }
    fn select(self: &Rc<Self>, link: Rc<CachedBrowserLink>) -> Result<(), JsValue> {
        let event = self.last_event.borrow().clone();
        let Some(event) = event else { return Ok(()) };
        let Some(position) = (self.coordinates)(&event)? else {
            return Ok(());
        };
        if link.contains_with_columns(position, || self.cols.get())? {
            // Assign current before invoking decorations getters, just as source.
            self.hover
                .borrow_mut()
                .begin_enter(link.clone(), Default::default());
            let values = link.decorations()?;
            let underline = values.0.is_truthy();
            *link.decoration_values.borrow_mut() = Some(values);
            link.hovered.set(true);
            // Source samples _lastMouseEvent on entry to _linkHover, after
            // decorations getters but before underline callbacks. Reentry can
            // replace it, and disposal can clear it to undefined.
            let event = self
                .last_event
                .borrow()
                .clone()
                .map(JsValue::from)
                .unwrap_or(JsValue::UNDEFINED);
            if underline {
                self.fire_underline(&link, true)?;
            }
            if self.current_pointer()? {
                self.element.class_list().add_1("xterm-cursor-pointer")?;
            }
            link.invoke("hover", &event)?;
            self.install_decorations(&link)?;
        }
        Ok(())
    }
    fn current_pointer(&self) -> Result<bool, JsValue> {
        let current = self.hover.borrow().current().cloned().ok_or_else(|| {
            js_sys::TypeError::new("Cannot read properties of undefined (reading 'state')")
        })?;
        let values = current.decoration_values.borrow();
        Ok(values
            .as_ref()
            .ok_or_else(|| {
                js_sys::TypeError::new(
                    "Cannot read properties of undefined (reading 'decorations')",
                )
            })?
            .1
            .is_truthy())
    }
    pub(super) fn fire_underline(&self, link: &BrowserLink, show: bool) -> Result<(), JsValue> {
        let (cols, offset) = self.hover.borrow().geometry();
        (self.underline)(show, link.underline_event(cols, offset)?)
    }
}
