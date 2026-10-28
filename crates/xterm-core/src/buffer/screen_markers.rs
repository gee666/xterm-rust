// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{Buffer, BufferError};
use crate::{buffer::marker::Marker, event::EmitError};
use std::rc::Rc;

fn marker_overflow() -> EmitError {
    EmitError::callback(BufferError::SizeOverflow)
}

impl Buffer {
    pub fn clear_markers(&mut self, y: i64) -> Result<(), BufferError> {
        self.is_clearing.set(true);
        let mut index = 0;
        loop {
            let marker = self.markers.borrow().get(index).cloned();
            let Some(marker) = marker else {
                break;
            };
            if marker.line() == y {
                marker.dispose()?;
                if index < self.markers.borrow().len() {
                    self.markers.borrow_mut().remove(index);
                }
            } else {
                index += 1;
            }
        }
        self.is_clearing.set(false);
        Ok(())
    }
    pub fn clear_all_markers(&mut self) -> Result<(), BufferError> {
        super::shared::clear_markers(&self.markers, &self.is_clearing)
    }
    pub fn add_marker(&mut self, y: i64) -> Marker {
        let marker = Marker::new(y);
        self.markers.borrow_mut().push(marker.clone());
        let weak = marker.downgrade();
        marker.register(
            self.lines
                .on_trim_emitter
                .event()
                .try_subscribe(move |amount| {
                    if let Some(marker) = weak.upgrade() {
                        let amount = i64::try_from(*amount).map_err(|_| marker_overflow())?;
                        marker.set_line(
                            marker
                                .line()
                                .checked_sub(amount)
                                .ok_or_else(marker_overflow)?,
                        );
                        if marker.line() < 0 {
                            marker.dispose()?;
                        }
                    }
                    Ok(())
                }),
        );
        let weak = marker.downgrade();
        marker.register(
            self.lines
                .on_insert_emitter
                .event()
                .try_subscribe(move |event| {
                    if let Some(marker) = weak.upgrade() {
                        let index = i64::try_from(event.index).map_err(|_| marker_overflow())?;
                        if marker.line() >= index {
                            let amount =
                                i64::try_from(event.amount).map_err(|_| marker_overflow())?;
                            marker.set_line(
                                marker
                                    .line()
                                    .checked_add(amount)
                                    .ok_or_else(marker_overflow)?,
                            );
                        }
                    }
                    Ok(())
                }),
        );
        let weak = marker.downgrade();
        marker.register(
            self.lines
                .on_delete_emitter
                .event()
                .try_subscribe(move |event| {
                    if let Some(marker) = weak.upgrade() {
                        let index = i64::try_from(event.index).map_err(|_| marker_overflow())?;
                        let amount = i64::try_from(event.amount).map_err(|_| marker_overflow())?;
                        let end = index.checked_add(amount).ok_or_else(marker_overflow)?;
                        if marker.line() >= index && marker.line() < end {
                            marker.dispose()?;
                        }
                        if marker.line() > index {
                            marker.set_line(
                                marker
                                    .line()
                                    .checked_sub(amount)
                                    .ok_or_else(marker_overflow)?,
                            );
                        }
                    }
                    Ok(())
                }),
        );
        let markers = Rc::downgrade(&self.markers);
        let clearing = Rc::downgrade(&self.is_clearing);
        let weak = marker.downgrade();
        marker.register(marker.on_dispose().subscribe(move |_| {
            if clearing.upgrade().is_some_and(|flag| flag.get()) {
                return;
            }
            if let (Some(markers), Some(marker)) = (markers.upgrade(), weak.upgrade()) {
                let mut values = markers.borrow_mut();
                let index = values.iter().position(|value| value.ptr_eq(&marker));
                if let Some(index) = index {
                    values.remove(index);
                } else {
                    values.pop();
                }
            }
        }));
        marker
    }
}
