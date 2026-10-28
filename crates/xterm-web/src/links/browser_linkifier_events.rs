// Copyright (c) 2019-2026 The xterm.js authors. All rights reserved. MIT license.
use super::State;
use crate::links::{HoverEffect, Link, Position, ProviderRequest};
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, MouseEvent};

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
        } else if !self.hover.borrow().contains(position) {
            self.clear_current()?;
            self.ask(position, true)?;
        }
        self.last_cell.set(Some(position));
        Ok(())
    }
    pub(super) fn clear_current(&self) -> Result<(), JsValue> {
        if self.last_event.borrow().is_none() {
            return Ok(());
        }
        let effects = self.hover.borrow_mut().leave();
        self.apply(effects, None)?;
        self.hover.borrow_mut().clear_current();
        Ok(())
    }
    pub(super) fn ask(self: &Rc<Self>, position: Position, use_cache: bool) -> Result<(), JsValue> {
        if !use_cache || self.active_line.get().is_none() {
            let old = self.replies.borrow().cached_links();
            let handlers: Vec<_> = old
                .iter()
                .filter_map(|data| self.handlers.borrow().get(&Rc::as_ptr(data)).cloned())
                .collect();
            for handler in handlers {
                if let Some(dispose) = &handler.dispose {
                    dispose()?;
                }
            }
            self.replies.borrow_mut().clear();
            let handlers = std::mem::take(&mut *self.handlers.borrow_mut());
            drop(handlers);
            self.active_line.set(Some(position.y));
        }
        let request = ProviderRequest::new(position);
        let mut index = 0;
        while let Some(provider) = self.providers.get(index) {
            if use_cache {
                let selected = self.replies.borrow().from_cache_index(
                    index,
                    &request,
                    self.providers.len(),
                    self.cols.get(),
                );
                if let Some(selected) = selected {
                    self.select(selected)?;
                }
            } else {
                let weak = Rc::downgrade(self);
                let request = request.clone();
                provider.provide_links(
                    position.y,
                    Box::new(move |links| {
                        let Some(state) = weak
                            .upgrade()
                            .filter(|s| !s.disposed.get() && !s.mouse_out.get())
                        else {
                            return Ok(());
                        };
                        let data = links.map(|links| {
                            links
                                .into_iter()
                                .map(|link| {
                                    let data = link.data.clone();
                                    state
                                        .handlers
                                        .borrow_mut()
                                        .insert(Rc::as_ptr(&data), Rc::new(link));
                                    data
                                })
                                .collect()
                        });
                        let selected = state.replies.borrow_mut().receive_unfiltered(
                            index,
                            data,
                            &request,
                            state.providers.len(),
                            state.cols.get(),
                        );
                        if let Some(selected) = selected {
                            state.select(selected)?;
                        }
                        state.replies.borrow_mut().filter_complete(
                            request.position.y,
                            state.providers.len(),
                            state.cols.get(),
                        );
                        Ok(())
                    }),
                )?;
            }
            index += 1;
        }
        Ok(())
    }
    fn select(&self, data: Rc<Link>) -> Result<(), JsValue> {
        let event = self.last_event.borrow().clone();
        let position = match event.as_ref() {
            Some(event) => (self.coordinates)(event)?,
            None => None,
        };
        let Some(position) = position else {
            return Ok(());
        };
        if data.range.contains(position, self.cols.get()) {
            let decorations = self
                .handlers
                .borrow()
                .get(&Rc::as_ptr(&data))
                .and_then(|link| link.decorations);
            let effects = self.hover.borrow_mut().enter(data, decorations);
            self.apply(effects, None)?;
        }
        Ok(())
    }
    pub(super) fn apply(
        &self,
        effects: Vec<HoverEffect>,
        event: Option<&MouseEvent>,
    ) -> Result<(), JsValue> {
        let transition = effects
            .iter()
            .any(|effect| matches!(effect, HoverEffect::Hover(_) | HoverEffect::Leave(_)));
        let mouse_event = event.cloned().or_else(|| self.last_event.borrow().clone());
        // A callback can replace the row cache during an underline event. Hold
        // the original link's callbacks just as the source's local link does.
        let effects: Vec<_> = effects
            .into_iter()
            .map(|effect| {
                let data = match &effect {
                    HoverEffect::Hover(data)
                    | HoverEffect::Leave(data)
                    | HoverEffect::Activate(data) => Some(data),
                    _ => None,
                };
                let handler = data
                    .map(|data| {
                        self.handlers
                            .borrow()
                            .get(&Rc::as_ptr(data))
                            .cloned()
                            .ok_or_else(|| JsValue::from_str("link callback identity missing"))
                    })
                    .transpose()?;
                Ok((effect, handler))
            })
            .collect::<Result<_, JsValue>>()?;
        for (effect, handler) in effects {
            match &effect {
                HoverEffect::Underline { show, event } => (self.underline)(*show, *event)?,
                HoverEffect::Pointer(enabled) if !transition => {
                    self.element
                        .class_list()
                        .toggle_with_force("xterm-cursor-pointer", *enabled)?;
                }
                HoverEffect::Pointer(_) => {
                    // Hover/leave re-read pointer state below, after underline
                    // listeners have had the opportunity to change decorations.
                }
                HoverEffect::Hover(data)
                | HoverEffect::Leave(data)
                | HoverEffect::Activate(data) => {
                    if !matches!(effect, HoverEffect::Activate(_)) {
                        let decorations = self.hover.borrow().decorations().ok_or_else(|| {
                            js_sys::TypeError::new(
                                "Cannot read properties of undefined (reading 'state')",
                            )
                        })?;
                        if decorations.pointer_cursor {
                            self.element.class_list().toggle_with_force(
                                "xterm-cursor-pointer",
                                matches!(effect, HoverEffect::Hover(_)),
                            )?;
                        }
                    }
                    // This variant always carries its original callback identity.
                    let handler = handler
                        .ok_or_else(|| JsValue::from_str("link callback identity missing"))?;
                    let callback = match &effect {
                        HoverEffect::Hover(_) => handler.hover.clone(),
                        HoverEffect::Leave(_) => handler.leave.clone(),
                        HoverEffect::Activate(_) => Some(handler.activate.clone()),
                        _ => unreachable!("link callback effects only"),
                    };
                    if let (Some(callback), Some(event)) = (callback, mouse_event.as_ref()) {
                        callback(event, data)?;
                    }
                }
            }
        }
        Ok(())
    }
}
