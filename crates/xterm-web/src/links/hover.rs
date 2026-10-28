// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use super::{Link, Position, UnderlineEvent};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Decorations {
    pub underline: bool,
    pub pointer_cursor: bool,
}
impl Default for Decorations {
    fn default() -> Self {
        Self {
            underline: true,
            pointer_cursor: true,
        }
    }
}

/// Host effects are returned in callback order, never called under a state borrow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HoverEffect {
    Underline { show: bool, event: UnderlineEvent },
    Pointer(bool),
    Hover(Rc<Link>),
    Leave(Rc<Link>),
    Activate(Rc<Link>),
}

pub struct HoverState {
    current: Option<Rc<Link>>,
    mouse_down: Option<Rc<Link>>,
    decorations: Decorations,
    hovered: bool,
    cols: usize,
    scroll_offset: usize,
}
impl HoverState {
    pub fn new(cols: usize, scroll_offset: usize) -> Self {
        Self {
            current: None,
            mouse_down: None,
            decorations: Decorations::default(),
            hovered: false,
            cols,
            scroll_offset,
        }
    }
    pub fn current(&self) -> Option<&Rc<Link>> {
        self.current.as_ref()
    }
    pub fn set_geometry(&mut self, cols: usize, scroll_offset: usize) {
        self.cols = cols;
        self.scroll_offset = scroll_offset;
    }
    pub fn contains(&self, position: Position) -> bool {
        self.current
            .as_ref()
            .is_some_and(|link| link.range.contains(position, self.cols))
    }
    pub fn enter(&mut self, link: Rc<Link>, decorations: Option<Decorations>) -> Vec<HoverEffect> {
        self.current = Some(Rc::clone(&link));
        self.decorations = decorations.unwrap_or_default();
        self.hovered = true;
        let mut effects = Vec::new();
        if self.decorations.underline {
            effects.push(HoverEffect::Underline {
                show: true,
                event: link.underline_event(self.cols, self.scroll_offset),
            });
        }
        if self.decorations.pointer_cursor {
            effects.push(HoverEffect::Pointer(true));
        }
        effects.push(HoverEffect::Hover(link));
        effects
    }
    pub fn leave(&mut self) -> Vec<HoverEffect> {
        let Some(link) = self.current.clone() else {
            return Vec::new();
        };
        self.hovered = false;
        let mut effects = Vec::new();
        if self.decorations.underline {
            effects.push(HoverEffect::Underline {
                show: false,
                event: link.underline_event(self.cols, self.scroll_offset),
            });
        }
        if self.decorations.pointer_cursor {
            effects.push(HoverEffect::Pointer(false));
        }
        effects.push(HoverEffect::Leave(link));
        effects
    }
    /// Call after leave effects succeed. A throwing leave callback keeps the
    /// current link visible to subsequent calls, as upstream does.
    pub fn clear_current(&mut self) {
        self.current = None;
    }
    pub fn mouse_down(&mut self) {
        self.mouse_down.clone_from(&self.current);
    }
    pub fn mouse_up(&self, position: Option<Position>) -> Option<HoverEffect> {
        let link = self.current.as_ref()?;
        if self.mouse_down.as_ref()?.as_ref() == link.as_ref() && self.contains(position?) {
            Some(HoverEffect::Activate(Rc::clone(link)))
        } else {
            None
        }
    }
    pub fn set_underline(&mut self, value: bool) -> Option<HoverEffect> {
        if self.decorations.underline == value {
            return None;
        }
        self.decorations.underline = value;
        self.current
            .as_ref()
            .filter(|_| self.hovered)
            .map(|link| HoverEffect::Underline {
                show: value,
                event: link.underline_event(self.cols, self.scroll_offset),
            })
    }
    pub fn set_pointer_cursor(&mut self, value: bool) -> Option<HoverEffect> {
        if self.decorations.pointer_cursor == value {
            return None;
        }
        self.decorations.pointer_cursor = value;
        self.current
            .as_ref()
            .filter(|_| self.hovered)
            .map(|_| HoverEffect::Pointer(value))
    }
    /// The source invalidates a link only if its full range is contained in the
    /// rendered buffer rows. Start zero also clears links above the viewport.
    pub fn invalidated_by_render(&self, start: usize, end: usize) -> bool {
        let start = if start == 0 {
            0
        } else {
            start + 1 + self.scroll_offset
        };
        let end = self.scroll_offset + 1 + end;
        self.current
            .as_ref()
            .is_some_and(|link| link.range.start.y >= start && link.range.end.y <= end)
    }
}
