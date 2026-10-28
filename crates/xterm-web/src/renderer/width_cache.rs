// Copyright (c) 2023 The xterm.js authors. All rights reserved. MIT license.
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt,
    rc::{Rc, Weak},
};
pub const FLAT_UNSET: f32 = -9999.0;
pub const FLAT_SIZE: usize = 256;
#[derive(Clone, Debug, PartialEq)]
pub struct Font {
    pub family: String,
    pub size: f64,
    pub weight: String,
    pub weight_bold: String,
}
impl Default for Font {
    fn default() -> Self {
        Self {
            family: String::new(),
            size: 0.0,
            weight: "normal".into(),
            weight_bold: "bold".into(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum FontVariant {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}
impl FontVariant {
    pub const ALL: [Self; 4] = [Self::Regular, Self::Bold, Self::Italic, Self::BoldItalic];
    pub fn is_bold(self) -> bool {
        matches!(self, Self::Bold | Self::BoldItalic)
    }
    pub fn is_italic(self) -> bool {
        matches!(self, Self::Italic | Self::BoldItalic)
    }
}
/// Key equality must be pure strict equality, not a font conversion. Callbacks
/// use shared receivers and run without any cache borrow.
pub trait WidthMeasurer {
    type Font: Default + PartialEq;
    type Error;
    fn configure_variant(&self, font: &Self::Font, variant: FontVariant)
        -> Result<(), Self::Error>;
    fn measure(&self, text: &[u16], variant: FontVariant) -> Result<f64, Self::Error>;
}
#[derive(Debug, PartialEq)]
pub enum WidthError<E> {
    Disposed,
    Measure(E),
}
impl<E: fmt::Display> fmt::Display for WidthError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disposed => f.write_str("width measurement canvas has been disposed"),
            Self::Measure(e) => e.fmt(f),
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for WidthError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Measure(e) => Some(e),
            Self::Disposed => None,
        }
    }
}
struct State<M: WidthMeasurer> {
    flat: [f32; FLAT_SIZE],
    holey: Option<HashMap<Vec<u16>, f64>>,
    font: Rc<M::Font>,
    measurer: Option<Rc<M>>,
}
/// Clones share state. Use a weak handle in a measurer callback to avoid a cycle.
pub struct WidthCache<M: WidthMeasurer> {
    state: Rc<RefCell<State<M>>>,
}
pub struct WeakWidthCache<M: WidthMeasurer> {
    state: Weak<RefCell<State<M>>>,
}
impl<M: WidthMeasurer> Clone for WidthCache<M> {
    fn clone(&self) -> Self {
        Self {
            state: Rc::clone(&self.state),
        }
    }
}
impl<M: WidthMeasurer> Clone for WeakWidthCache<M> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}
impl<M: WidthMeasurer> WeakWidthCache<M> {
    pub fn upgrade(&self) -> Option<WidthCache<M>> {
        self.state.upgrade().map(|state| WidthCache { state })
    }
}
impl<M: WidthMeasurer> WidthCache<M> {
    pub fn new(measurer: M) -> Self {
        Self {
            state: Rc::new(RefCell::new(State {
                flat: [FLAT_UNSET; FLAT_SIZE],
                holey: Some(HashMap::new()),
                font: Rc::new(M::Font::default()),
                measurer: Some(Rc::new(measurer)),
            })),
        }
    }
    pub fn downgrade(&self) -> WeakWidthCache<M> {
        WeakWidthCache {
            state: Rc::downgrade(&self.state),
        }
    }
    pub fn dispose(&self) {
        let removed = {
            let mut s = self.state.borrow_mut();
            (s.measurer.take(), s.holey.take())
        };
        drop(removed);
    }
    pub fn clear(&self) {
        let mut s = self.state.borrow_mut();
        s.flat.fill(FLAT_UNSET);
        s.holey = Some(HashMap::new());
    }
    pub fn set_font(&self, font: M::Font) -> Result<(), WidthError<M::Error>> {
        let previous = Rc::clone(&self.state.borrow().font);
        if *previous == font {
            return Ok(());
        }
        let font = Rc::new(font);
        self.state.borrow_mut().font = Rc::clone(&font);
        for variant in FontVariant::ALL {
            // Recheck availability after each callback. A retained measurer may
            // finish its current call after disposal, but cannot start the next.
            self.measurer()?
                .configure_variant(&font, variant)
                .map_err(WidthError::Measure)?;
        }
        self.clear();
        Ok(())
    }
    pub fn get(&self, text: &[u16], bold: bool, italic: bool) -> Result<f64, WidthError<M::Error>> {
        if !bold && !italic && text.len() == 1 && usize::from(text[0]) < FLAT_SIZE {
            let index = usize::from(text[0]);
            let cached = self.state.borrow().flat[index];
            if cached != FLAT_UNSET {
                return Ok(f64::from(cached));
            }
            let width = self
                .measurer()?
                .measure(text, FontVariant::Regular)
                .map_err(WidthError::Measure)?;
            if width > 0.0 {
                self.state.borrow_mut().flat[index] = width as f32;
            }
            return Ok(width);
        }
        let mut key = text.to_vec();
        if bold {
            key.push(u16::from(b'B'));
        }
        if italic {
            key.push(u16::from(b'I'));
        }
        let cached = {
            let s = self.state.borrow();
            s.holey
                .as_ref()
                .ok_or(WidthError::Disposed)?
                .get(&key)
                .copied()
        };
        if let Some(width) = cached {
            return Ok(width);
        }
        let variant = match (bold, italic) {
            (false, false) => FontVariant::Regular,
            (true, false) => FontVariant::Bold,
            (false, true) => FontVariant::Italic,
            (true, true) => FontVariant::BoldItalic,
        };
        let width = self
            .measurer()?
            .measure(text, variant)
            .map_err(WidthError::Measure)?;
        if width > 0.0 {
            // clear/setFont can replace the map while measuring. Source stores
            // into the current map, and throws if a callback disposed it.
            self.state
                .borrow_mut()
                .holey
                .as_mut()
                .ok_or(WidthError::Disposed)?
                .insert(key, width);
        }
        Ok(width)
    }
    fn measurer(&self) -> Result<Rc<M>, WidthError<M::Error>> {
        self.state
            .borrow()
            .measurer
            .as_ref()
            .cloned()
            .ok_or(WidthError::Disposed)
    }
}
#[cfg(test)]
#[path = "width_cache_reentry_tests.rs"]
mod reentry_tests;
#[cfg(test)]
#[path = "width_cache_tests.rs"]
mod tests;
