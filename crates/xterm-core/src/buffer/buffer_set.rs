// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Normal/alternate buffer ownership and synchronous activation events.
use super::{AttributeData, Buffer, BufferDimensions, BufferError};
use crate::{
    event::{EmitError, Emitter, Event},
    lifecycle::{Disposable, DisposableHandle},
    options::OptionsService,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub type BufferRef = Rc<RefCell<Buffer>>;

#[derive(Clone)]
pub struct BufferActivation {
    pub active_buffer: BufferRef,
    pub inactive_buffer: BufferRef,
}

struct State {
    normal: RefCell<BufferRef>,
    alt: RefCell<BufferRef>,
    active: RefCell<BufferRef>,
    dimensions: Rc<BufferDimensions>,
    options: Rc<OptionsService>,
    activate: Emitter<BufferActivation>,
    disposed: Cell<bool>,
}

/// Handles are stable until reset; activation changes only the active handle.
/// No ownership borrow is held while activation callbacks run.
pub struct BufferSet {
    state: Rc<State>,
    option_listeners: Vec<DisposableHandle>,
}

impl BufferSet {
    pub fn new(cols: usize, rows: usize, options: Rc<OptionsService>) -> Result<Self, BufferError> {
        let dimensions = Rc::new(BufferDimensions::new(cols, rows));
        let normal = make_buffer(true, &dimensions, &options)?;
        normal.borrow_mut().fill_viewport_rows(None)?;
        let alt = make_buffer(false, &dimensions, &options)?;
        let state = Rc::new(State {
            active: RefCell::new(Rc::clone(&normal)),
            normal: RefCell::new(normal),
            alt: RefCell::new(alt),
            dimensions,
            options: Rc::clone(&options),
            activate: Emitter::new(),
            disposed: Cell::new(false),
        });
        let mut set = Self {
            state,
            option_listeners: Vec::new(),
        };
        set.fire_activation(set.normal(), set.alt())?;
        set.setup_tab_stops(None)?;
        for key in ["scrollback", "tabStopWidth"] {
            let weak = Rc::downgrade(&set.state);
            set.option_listeners
                .push(options.on_option_change().try_subscribe(move |changed| {
                    if *changed != key {
                        return Ok(());
                    }
                    let Some(state) = weak.upgrade() else {
                        return Ok(());
                    };
                    let result = if key == "scrollback" {
                        resize_state(
                            &state,
                            state.dimensions.cols.get(),
                            state.dimensions.rows.get(),
                        )
                    } else {
                        setup_tabs(&state, None)
                    };
                    result.map_err(EmitError::callback)
                }));
        }
        Ok(set)
    }
    pub fn normal(&self) -> BufferRef {
        Rc::clone(&self.state.normal.borrow())
    }
    pub fn alt(&self) -> BufferRef {
        Rc::clone(&self.state.alt.borrow())
    }
    pub fn active(&self) -> BufferRef {
        Rc::clone(&self.state.active.borrow())
    }
    pub fn dimensions(&self) -> Rc<BufferDimensions> {
        Rc::clone(&self.state.dimensions)
    }
    pub fn on_buffer_activate(&self) -> Event<BufferActivation> {
        self.state.activate.event()
    }

    pub fn reset(&self) -> Result<(), BufferError> {
        let normal = make_buffer(true, &self.state.dimensions, &self.state.options)?;
        let old = self.state.normal.replace(Rc::clone(&normal));
        if !self.state.disposed.get() {
            old.try_borrow_mut()
                .map_err(|_| BufferError::BorrowedLine)?
                .dispose()?;
        }
        normal.borrow_mut().fill_viewport_rows(None)?;
        let alt = make_buffer(false, &self.state.dimensions, &self.state.options)?;
        let old = self.state.alt.replace(Rc::clone(&alt));
        if !self.state.disposed.get() {
            old.try_borrow_mut()
                .map_err(|_| BufferError::BorrowedLine)?
                .dispose()?;
        }
        *self.state.active.borrow_mut() = Rc::clone(&normal);
        self.fire_activation(normal, alt)?;
        self.setup_tab_stops(None)
    }
    pub fn activate_normal_buffer(&self) -> Result<(), BufferError> {
        let normal = self.normal();
        if Rc::ptr_eq(&normal, &self.active()) {
            return Ok(());
        }
        let alt = self.alt();
        {
            let alt = alt.try_borrow().map_err(|_| BufferError::BorrowedLine)?;
            let mut normal = normal
                .try_borrow_mut()
                .map_err(|_| BufferError::BorrowedLine)?;
            normal.x = alt.x;
            normal.y = alt.y;
        }
        {
            let mut alt = alt
                .try_borrow_mut()
                .map_err(|_| BufferError::BorrowedLine)?;
            alt.clear_all_markers()?;
            alt.clear()?;
        }
        *self.state.active.borrow_mut() = Rc::clone(&normal);
        self.fire_activation(normal, alt)
    }
    pub fn activate_alt_buffer(
        &self,
        fill_attr: Option<&AttributeData>,
    ) -> Result<(), BufferError> {
        let alt = self.alt();
        if Rc::ptr_eq(&alt, &self.active()) {
            return Ok(());
        }
        let normal = self.normal();
        {
            let mut alt = alt
                .try_borrow_mut()
                .map_err(|_| BufferError::BorrowedLine)?;
            alt.fill_viewport_rows(fill_attr)?;
            let normal = normal.try_borrow().map_err(|_| BufferError::BorrowedLine)?;
            alt.x = normal.x;
            alt.y = normal.y;
        }
        *self.state.active.borrow_mut() = Rc::clone(&alt);
        self.fire_activation(alt, normal)
    }
    pub fn resize(&self, cols: usize, rows: usize) -> Result<(), BufferError> {
        resize_state(&self.state, cols, rows)
    }
    pub fn setup_tab_stops(&self, start: Option<usize>) -> Result<(), BufferError> {
        setup_tabs(&self.state, start)
    }
    fn fire_activation(
        &self,
        active_buffer: BufferRef,
        inactive_buffer: BufferRef,
    ) -> Result<(), BufferError> {
        self.state.activate.fire(&BufferActivation {
            active_buffer,
            inactive_buffer,
        })?;
        Ok(())
    }
    /// Explicit disposal propagates marker callback failures; native Drop only
    /// unregisters host option listeners, since it cannot return callback errors.
    pub fn dispose(&self) -> Result<(), BufferError> {
        if self.state.disposed.replace(true) {
            return Ok(());
        }
        self.normal()
            .try_borrow_mut()
            .map_err(|_| BufferError::BorrowedLine)?
            .dispose()?;
        self.alt()
            .try_borrow_mut()
            .map_err(|_| BufferError::BorrowedLine)?
            .dispose()?;
        self.state.activate.dispose();
        for listener in &self.option_listeners {
            listener.dispose();
        }
        Ok(())
    }
}
impl Drop for BufferSet {
    fn drop(&mut self) {
        for listener in &self.option_listeners {
            listener.dispose();
        }
    }
}
fn make_buffer(
    has_scrollback: bool,
    dimensions: &Rc<BufferDimensions>,
    options: &Rc<OptionsService>,
) -> Result<BufferRef, BufferError> {
    let mut buffer = Buffer::new(
        has_scrollback,
        dimensions.cols.get(),
        dimensions.rows.get(),
        Rc::clone(options),
    )?;
    buffer.host_dimensions = Rc::clone(dimensions);
    Ok(Rc::new(RefCell::new(buffer)))
}
fn resize_state(state: &State, cols: usize, rows: usize) -> Result<(), BufferError> {
    let normal = Rc::clone(&state.normal.borrow());
    normal
        .try_borrow_mut()
        .map_err(|_| BufferError::BorrowedLine)?
        .resize(cols, rows)?;
    let alt = Rc::clone(&state.alt.borrow());
    alt.try_borrow_mut()
        .map_err(|_| BufferError::BorrowedLine)?
        .resize(cols, rows)?;
    setup_tabs(state, Some(cols))
}
fn setup_tabs(state: &State, start: Option<usize>) -> Result<(), BufferError> {
    let normal = Rc::clone(&state.normal.borrow());
    normal
        .try_borrow_mut()
        .map_err(|_| BufferError::BorrowedLine)?
        .setup_tab_stops(start)?;
    let alt = Rc::clone(&state.alt.borrow());
    alt.try_borrow_mut()
        .map_err(|_| BufferError::BorrowedLine)?
        .setup_tab_stops(start)?;
    Ok(())
}
