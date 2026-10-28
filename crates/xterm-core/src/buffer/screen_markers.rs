// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{Buffer, BufferError};
use crate::{
    buffer::{marker::Marker, marker_line::MarkerLine},
    event::EmitError,
};
use std::rc::Rc;

impl Buffer {
    pub fn clear_markers(&mut self, y: i64) -> Result<(), BufferError> {
        self.clear_markers_value(&MarkerLine::from(y))
    }

    pub fn clear_markers_value(&mut self, row: &MarkerLine) -> Result<(), BufferError> {
        self.is_clearing.set(true);
        let mut index = 0;
        loop {
            let marker = self.markers.borrow().get(index).cloned();
            let Some(marker) = marker else {
                break;
            };
            if marker.line_value().strict_eq(row) {
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
        self.add_marker_value(MarkerLine::from(y))
    }

    /// The caller evaluates host coercion before borrowing the captured buffer.
    pub fn add_marker_value(&mut self, line: MarkerLine) -> Marker {
        let marker = Marker::new(line);
        self.markers.borrow_mut().push(marker.clone());
        let weak = marker.downgrade();
        marker.register(
            self.lines
                .on_trim_emitter
                .event()
                .try_subscribe(move |amount| {
                    if let Some(marker) = weak.upgrade() {
                        marker.subtract_from_line(*amount as f64);
                        if marker.line_number() < 0.0 {
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
                        if marker.line_number() >= event.index as f64 {
                            marker
                                .add_to_line(event.amount as f64)
                                .map_err(EmitError::callback)?;
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
                        let index = event.index as f64;
                        let amount = event.amount as f64;
                        if marker.line_number() >= index && marker.line_number() < index + amount {
                            marker.dispose()?;
                        }
                        if marker.line_number() > index {
                            marker.subtract_from_line(amount);
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
