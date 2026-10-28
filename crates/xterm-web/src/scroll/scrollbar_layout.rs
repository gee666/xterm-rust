// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Scrollbar geometry construction, dirty-state updates and CSS placement.
use super::{Axis, BrowserScrollable, Inner, ScrollbarState, SharedOptions, V};
use crate::scroll::{options::number, state::ScrollEvent, ScrollbarVisibility};
use wasm_bindgen::JsValue;

pub(super) struct Initial {
    pub(super) state: ScrollbarState,
    pub(super) has_arrows: V,
    pub(super) lazy_render: bool,
    pub(super) scroll_by_page: bool,
    pub(super) visibility: ScrollbarVisibility,
}
pub(super) fn prepare(
    axis: Axis,
    model: &BrowserScrollable,
    options: &SharedOptions,
) -> Result<Initial, JsValue> {
    let snapshot = model.scroll_state();
    let (has_arrows, lazy_render) = match axis {
        Axis::Horizontal => {
            let lazy = options.borrow().lazy_render.is_truthy();
            let arrows = options.borrow().horizontal_has_arrows.clone();
            (arrows, lazy)
        }
        // Vertical captures hasArrows before evaluating its super arguments.
        Axis::Vertical => {
            let arrows = options.borrow().vertical_has_arrows.clone();
            let lazy = options.borrow().lazy_render.is_truthy();
            (arrows, lazy)
        }
    };
    let arrow = if has_arrows.is_truthy() {
        let options = options.borrow();
        match axis {
            Axis::Horizontal => options.horizontal_scrollbar_size.clone(),
            Axis::Vertical => options.vertical_scrollbar_size.clone(),
        }
    } else {
        0.0.into()
    };
    let size = {
        let options = options.borrow();
        match axis {
            Axis::Horizontal => {
                if options.horizontal.is_hidden() {
                    0.0.into()
                } else {
                    options.horizontal_scrollbar_size.clone()
                }
            }
            Axis::Vertical => {
                if options.vertical.is_hidden() {
                    0.0.into()
                } else {
                    options.vertical_scrollbar_size.clone()
                }
            }
        }
    };
    let opposite = if axis == Axis::Horizontal {
        let options = options.borrow();
        if options.vertical.is_hidden() {
            0.0.into()
        } else {
            options.vertical_scrollbar_size.clone()
        }
    } else {
        0.0.into()
    };
    // Argument selection precedes coercion. The state constructor performs
    // Math.round on size, opposite size, then arrow size, not argument order.
    let size = number(&size)?;
    let opposite = number(&opposite)?;
    let arrow = number(&arrow)?;
    let (visible, scroll, position) = match axis {
        Axis::Horizontal => (snapshot.width, snapshot.scroll_width, snapshot.left),
        Axis::Vertical => (snapshot.height, snapshot.scroll_height, snapshot.top),
    };
    let state = ScrollbarState::new(arrow, size, opposite, visible, scroll, position);
    let visibility = {
        let options = options.borrow();
        match axis {
            Axis::Horizontal => options.horizontal.clone(),
            Axis::Vertical => options.vertical.clone(),
        }
    };
    let scroll_by_page = options.borrow().scroll_by_page.is_truthy();
    Ok(Initial {
        state,
        has_arrows,
        lazy_render,
        scroll_by_page,
        visibility,
    })
}
#[derive(Clone, Copy)]
enum Metric {
    ScrollSize,
    Position,
    VisibleSize,
}
impl Inner {
    pub(super) fn handle_scroll(&self, event: &ScrollEvent) -> Result<bool, JsValue> {
        let (size, position, visible) = match self.axis {
            Axis::Horizontal => (
                event.current.scroll_width,
                event.current.left,
                event.current.width,
            ),
            Axis::Vertical => (
                event.current.scroll_height,
                event.current.top,
                event.current.height,
            ),
        };
        for (kind, value) in [
            (Metric::ScrollSize, size),
            (Metric::Position, position),
            (Metric::VisibleSize, visible),
        ] {
            let changed = {
                let mut state = self.state.borrow_mut();
                match kind {
                    Metric::ScrollSize => state.set_scroll_size(value),
                    Metric::Position => state.set_scroll_position(value),
                    Metric::VisibleSize => state.set_visible_size(value),
                }
            };
            if changed {
                let needed = self.state.borrow().is_needed();
                self.visibility.set_is_needed(needed)?;
                self.should_render.set(true);
                if !self.lazy_render {
                    self.render()?;
                }
            }
        }
        Ok(self.should_render.get())
    }
    pub(super) fn render(&self) -> Result<(), JsValue> {
        if !self.should_render.replace(false) {
            return Ok(());
        }
        let (large, small) = {
            let state = self.state.borrow();
            (
                state.get_rectangle_large_size(),
                state.get_rectangle_small_size(),
            )
        };
        match self.axis {
            Axis::Horizontal => {
                self.node.set_width(&large)?;
                self.node.set_height(&small)?;
                self.node.set_left(&0.0)?;
                self.node.set_bottom(&0.0)?;
            }
            Axis::Vertical => {
                self.node.set_width(&small)?;
                self.node.set_height(&large)?;
                self.node.set_right(&0.0)?;
                self.node.set_top(&0.0)?;
            }
        }
        // Source reads slider geometry only after container DOM writes. A style
        // getter may have synchronously changed the scroll state in between.
        let (size, position) = {
            let state = self.state.borrow();
            (
                state.get_slider_size(),
                state.get_arrow_size() + state.get_slider_position(),
            )
        };
        match self.axis {
            Axis::Horizontal => {
                self.slider()?.set_width(&size)?;
                self.slider()?.set_left(&position)?;
            }
            Axis::Vertical => {
                self.slider()?.set_height(&size)?;
                self.slider()?.set_top(&position)?;
            }
        }
        Ok(())
    }
    pub(in crate::scroll) fn update_scrollbar_size(&self, size: &V) -> Result<(), JsValue> {
        match self.axis {
            Axis::Horizontal => self.slider()?.set_height(size)?,
            Axis::Vertical => self.slider()?.set_width(size)?,
        }
        let size = number(size)?;
        self.state.borrow_mut().set_scrollbar_size(size);
        self.should_render.set(true);
        if !self.lazy_render {
            self.render()?;
        }
        Ok(())
    }
}
