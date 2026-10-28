// Copyright (c) 2019-2026 The xterm.js authors. All rights reserved. MIT license.
use super::State;
use crate::links::{HoverEffect, Link, Position, ProviderRequest};
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, MouseEvent};

impl State {
    pub(super) fn mouse_move(self: &Rc<Self>, event: MouseEvent) -> Result<(), JsValue> {
        *self.last_event.borrow_mut() = Some(event.clone());
        let Some(position) = (self.coordinates)(&event) else {
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
        self.apply(effects, None)
    }
    pub(super) fn ask(self: &Rc<Self>, position: Position, use_cache: bool) -> Result<(), JsValue> {
        if !use_cache || self.active_line.get().is_none() {
            let old = self.replies.borrow_mut().clear();
            for data in old {
                let handler = self.handlers.borrow().get(&Rc::as_ptr(&data)).cloned();
                if let Some(dispose) = handler.and_then(|handler| handler.dispose.clone()) {
                    dispose()?;
                }
            }
            self.handlers.borrow_mut().clear();
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
        let Some(position) = event.as_ref().and_then(|e| (self.coordinates)(e)) else {
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
        for effect in effects {
            match &effect {
                HoverEffect::Underline { show, event } => (self.underline)(*show, *event)?,
                HoverEffect::Pointer(enabled) => {
                    self.element
                        .class_list()
                        .toggle_with_force("xterm-cursor-pointer", *enabled)?;
                }
                HoverEffect::Hover(data)
                | HoverEffect::Leave(data)
                | HoverEffect::Activate(data) => {
                    let handler = self
                        .handlers
                        .borrow()
                        .get(&Rc::as_ptr(data))
                        .cloned()
                        .ok_or_else(|| JsValue::from_str("link callback identity missing"))?;
                    let callback = match &effect {
                        HoverEffect::Hover(_) => handler.hover.clone(),
                        HoverEffect::Leave(_) => handler.leave.clone(),
                        HoverEffect::Activate(_) => Some(handler.activate.clone()),
                        _ => unreachable!("link callback effects only"),
                    };
                    let last = self.last_event.borrow().clone();
                    if let (Some(callback), Some(event)) = (callback, event.or(last.as_ref())) {
                        callback(event, data)?;
                    }
                }
            }
        }
        Ok(())
    }
}
