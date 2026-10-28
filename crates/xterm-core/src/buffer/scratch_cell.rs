// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Buffer scratch cells retain identity across calls; snapshots do not.
use super::{AttributeData, CellData, ExtendedAttrs};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub struct ScratchCell {
    cell: Rc<RefCell<CellData>>,
}
impl ScratchCell {
    pub(super) fn new(code: u32, width: i32) -> Self {
        Self {
            cell: Rc::new(RefCell::new(CellData {
                content: code | ((width as u32) << super::constants::content::WIDTH_SHIFT),
                ..CellData::default()
            })),
        }
    }
    pub fn same_identity(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.cell, &other.cell)
    }
    /// Copy current content and flags while sharing extended attributes, as a
    /// cell load does. The owned result does not track later numeric mutations.
    pub fn snapshot(&self) -> CellData {
        let cell = self.cell.borrow();
        CellData {
            content: cell.content,
            combined_data: cell.combined_data.clone(),
            attributes: AttributeData {
                fg: cell.fg,
                bg: cell.bg,
                extended: cell.extended.share(),
            },
        }
    }
    pub(super) fn configure(&self, attr: Option<&AttributeData>) {
        let extended = attr.map_or_else(ExtendedAttrs::default, |a| a.extended.share());
        let previous = {
            let mut cell = self.cell.borrow_mut();
            cell.fg = attr.map_or(0, |a| a.fg);
            cell.bg = attr.map_or(0, |a| a.bg);
            std::mem::replace(&mut cell.extended, extended)
        };
        // An opaque native payload may reenter from its destructor.
        drop(previous);
    }
    pub fn fg(&self) -> u32 {
        self.cell.borrow().fg
    }
    pub fn set_fg(&self, value: u32) {
        self.cell.borrow_mut().fg = value;
    }
    pub fn bg(&self) -> u32 {
        self.cell.borrow().bg
    }
    pub fn set_bg(&self, value: u32) {
        self.cell.borrow_mut().bg = value;
    }
    pub fn content(&self) -> u32 {
        self.cell.borrow().content
    }
    pub fn set_content(&self, value: u32) {
        self.cell.borrow_mut().content = value;
    }
    pub fn combined_data(&self) -> Vec<u16> {
        self.cell.borrow().combined_data.clone()
    }
    pub fn set_combined_data(&self, value: Vec<u16>) {
        self.cell.borrow_mut().combined_data = value;
    }
    pub fn extended(&self) -> ExtendedAttrs {
        self.cell.borrow().extended.share()
    }
    pub fn set_extended(&self, value: ExtendedAttrs) {
        let previous = {
            let mut cell = self.cell.borrow_mut();
            std::mem::replace(&mut cell.extended, value)
        };
        drop(previous);
    }
}
