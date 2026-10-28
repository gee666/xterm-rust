// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Overview zones retain shared identity across clear/reuse, as the source pool does.
use std::{
    cell::{BorrowError, BorrowMutError, RefCell},
    rc::Rc,
};
use xterm_core::decoration::{Decoration, OverviewPosition};

#[derive(Clone, Debug, PartialEq)]
pub struct ColorZone {
    pub color: Vec<u16>,
    pub position: Option<OverviewPosition>,
    pub start_buffer_line: f64,
    pub end_buffer_line: f64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColorZoneOptions {
    pub color: Vec<u16>,
    pub position: Option<OverviewPosition>,
}
/// Missing padding properties behave like JS undefined, disabling adjacency.
#[derive(Clone, Debug)]
pub struct ColorZonePadding {
    pub full: Option<f64>,
    pub left: Option<f64>,
    pub center: Option<f64>,
    pub right: Option<f64>,
}
impl Default for ColorZonePadding {
    fn default() -> Self {
        Self {
            full: Some(0.0),
            left: Some(0.0),
            center: Some(0.0),
            right: Some(0.0),
        }
    }
}
impl ColorZonePadding {
    pub fn uniform(padding: f64) -> Self {
        Self {
            full: Some(padding),
            left: Some(padding),
            center: Some(padding),
            right: Some(padding),
        }
    }
    fn get(&self, position: Option<OverviewPosition>) -> f64 {
        match position.unwrap_or(OverviewPosition::Full) {
            OverviewPosition::Full => self.full,
            OverviewPosition::Left => self.left,
            OverviewPosition::Center => self.center,
            OverviewPosition::Right => self.right,
        }
        .unwrap_or(f64::NAN)
    }
}
#[derive(Debug)]
pub enum ColorZoneError {
    Read(BorrowError),
    Write(BorrowMutError),
}
impl std::fmt::Display for ColorZoneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read(e) => e.fmt(f),
            Self::Write(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for ColorZoneError {}
impl From<BorrowError> for ColorZoneError {
    fn from(e: BorrowError) -> Self {
        Self::Read(e)
    }
}
impl From<BorrowMutError> for ColorZoneError {
    fn from(e: BorrowMutError) -> Self {
        Self::Write(e)
    }
}

pub type ColorZoneRef = Rc<RefCell<ColorZone>>;
pub type ColorZoneList = Rc<RefCell<Vec<ColorZoneRef>>>;
/// The returned zone list and individual zones remain the same objects between calls.
/// External mutation is supported. Release all borrows before invoking store methods.
pub struct ColorZoneStore {
    zones: ColorZoneList,
    pool: Vec<ColorZoneRef>,
    pool_index: usize,
    padding: Rc<RefCell<ColorZonePadding>>,
}
impl Default for ColorZoneStore {
    fn default() -> Self {
        Self::new()
    }
}
impl ColorZoneStore {
    pub fn new() -> Self {
        Self {
            zones: Rc::new(RefCell::new(Vec::new())),
            pool: Vec::new(),
            pool_index: 0,
            padding: Rc::new(RefCell::new(ColorZonePadding::default())),
        }
    }
    /// Reading zones trims unused pooled references, without changing the cursor.
    pub fn zones(&mut self) -> Result<ColorZoneList, ColorZoneError> {
        self.pool.truncate(self.zones.try_borrow()?.len());
        Ok(Rc::clone(&self.zones))
    }
    pub fn clear(&mut self) -> Result<(), ColorZoneError> {
        self.zones.try_borrow_mut()?.clear();
        self.pool_index = 0;
        Ok(())
    }
    pub fn set_padding(&mut self, padding: Rc<RefCell<ColorZonePadding>>) {
        self.padding = padding;
    }
    pub fn add_decoration(&mut self, decoration: &Decoration) -> Result<(), ColorZoneError> {
        let options = decoration.options();
        let options = options.try_borrow()?;
        let overview = options
            .overview_ruler_options
            .as_ref()
            .map(|overview| ColorZoneOptions {
                color: overview.color.encode_utf16().collect(),
                position: overview.position,
            });
        self.add(decoration.marker().line() as f64, overview.as_ref())
    }
    /// `None` is a decoration without overviewRulerOptions, not a default-color zone.
    pub fn add(
        &mut self,
        line: f64,
        options: Option<&ColorZoneOptions>,
    ) -> Result<(), ColorZoneError> {
        let Some(options) = options else {
            return Ok(());
        };
        for zone in self.zones.try_borrow()?.iter() {
            let mut zone = zone.try_borrow_mut()?;
            if zone.color != options.color || zone.position != options.position {
                continue;
            }
            if line >= zone.start_buffer_line && line <= zone.end_buffer_line {
                return Ok(());
            }
            let padding = self.padding.try_borrow()?.get(options.position);
            if line >= zone.start_buffer_line - padding && line <= zone.end_buffer_line + padding {
                zone.start_buffer_line = js_min(zone.start_buffer_line, line);
                zone.end_buffer_line = js_max(zone.end_buffer_line, line);
                // A bridge extends only the first matching zone. Never union later zones.
                return Ok(());
            }
        }
        let value = ColorZone {
            color: options.color.clone(),
            position: options.position,
            start_buffer_line: line,
            end_buffer_line: line,
        };
        let zone = if let Some(zone) = self.pool.get(self.pool_index) {
            *zone.try_borrow_mut()? = value;
            Rc::clone(zone)
        } else {
            Rc::new(RefCell::new(value))
        };
        self.zones.try_borrow_mut()?.push(Rc::clone(&zone));
        if self.pool_index >= self.pool.len() {
            self.pool.push(zone);
        }
        self.pool_index += 1;
        Ok(())
    }
}
fn js_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == 0.0 && b == 0.0 && (a.is_sign_negative() || b.is_sign_negative()) {
        -0.0
    } else {
        a.min(b)
    }
}
fn js_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == 0.0 && b == 0.0 && (a.is_sign_positive() || b.is_sign_positive()) {
        0.0
    } else {
        a.max(b)
    }
}
