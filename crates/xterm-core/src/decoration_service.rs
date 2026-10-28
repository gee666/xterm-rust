// Copyright (c) 2022 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use crate::circular_list::CircularList;
struct ServiceState {
    decorations: RefCell<DecorationList>,
    cache: DecorationLineCache,
    registered: Emitter<Decoration>,
    removed: Emitter<Decoration>,
    disposed: Cell<bool>,
    activation_listener: RefCell<Option<DisposableHandle>>,
}
impl Drop for ServiceState {
    fn drop(&mut self) {
        if let Some(listener) = self.activation_listener.get_mut().take() {
            listener.dispose();
        }
    }
}
/// Connect this service to the active buffer's lines at construction and activation.
/// Host microtask scheduling is driven by `line_cache().on_sync_requested()`.
#[derive(Clone)]
pub struct DecorationService {
    state: Rc<ServiceState>,
}
impl Default for DecorationService {
    fn default() -> Self {
        Self::new()
    }
}
impl DecorationService {
    pub fn new() -> Self {
        Self {
            state: Rc::new(ServiceState {
                decorations: RefCell::new(SortedList::from_reader(DecorationLineKey)),
                cache: DecorationLineCache::new(),
                registered: Emitter::new(),
                removed: Emitter::new(),
                disposed: Cell::new(false),
                activation_listener: RefCell::new(None),
            }),
        }
    }
    /// Attach to the live buffer set and follow normal/alternate activation.
    pub fn with_buffers(buffers: &crate::buffer::BufferSet) -> Result<Self, EmitError> {
        let service = Self::new();
        let weak = Rc::downgrade(&service.state);
        let listener = buffers.on_buffer_activate().try_subscribe(move |event| {
            if let Some(state) = weak.upgrade() {
                let buffer = event
                    .active_buffer
                    .try_borrow()
                    .map_err(EmitError::callback)?;
                state.cache.attach_to_buffer_lines(&buffer.lines);
            }
            Ok(())
        });
        *service.state.activation_listener.borrow_mut() = Some(listener);
        let active = buffers.active();
        let buffer = active.try_borrow().map_err(EmitError::callback)?;
        service.attach_to_buffer_lines(&buffer.lines);
        Ok(service)
    }
    pub fn attach_to_buffer_lines<T: Clone + 'static>(&self, lines: &CircularList<T>) {
        self.state.cache.attach_to_buffer_lines(lines);
    }
    pub fn line_cache(&self) -> DecorationLineCache {
        self.state.cache.clone()
    }
    pub fn on_decoration_registered(&self) -> Event<Decoration> {
        self.state.registered.event()
    }
    pub fn on_decoration_removed(&self) -> Event<Decoration> {
        self.state.removed.event()
    }
    pub fn decorations(&self) -> Result<std::vec::IntoIter<Decoration>, DecorationError> {
        Ok(self.state.decorations.borrow_mut().values()?)
    }
    pub fn register_decoration(
        &self,
        options: Rc<RefCell<DecorationOptions>>,
    ) -> Result<Option<Decoration>, DecorationError> {
        if options.borrow().marker.is_disposed() {
            return Ok(None);
        }
        let decoration = Decoration::new(options);
        let start = decoration.marker().line_value();
        if !start.rel_lt(&MarkerLine::number(0.0)) {
            span::validate(&start, decoration.height())?;
        }
        let weak = Rc::downgrade(&decoration.state);
        *decoration.state.marker_listener.borrow_mut() =
            Some(decoration.marker().on_dispose().try_subscribe(move |_| {
                if let Some(state) = weak.upgrade() {
                    Decoration { state }.dispose()?;
                }
                Ok(())
            }));
        let weak_service = Rc::downgrade(&self.state);
        let weak_decoration = Rc::downgrade(&decoration.state);
        *decoration.state.service_listener.borrow_mut() =
            Some(decoration.on_dispose().try_subscribe(move |_| {
                let Some(state) = weak_decoration.upgrade() else {
                    return Ok(());
                };
                let decoration = Decoration { state };
                let listener = decoration.state.service_listener.borrow_mut().take();
                if let Some(listener) = listener {
                    listener.dispose();
                }
                if let Some(service) = weak_service.upgrade() {
                    let removed = service
                        .decorations
                        .borrow_mut()
                        .delete_by(&decoration, Decoration::ptr_eq)
                        .map_err(EmitError::callback)?;
                    if removed {
                        service
                            .cache
                            .remove(&decoration)
                            .map_err(EmitError::callback)?;
                        service.removed.fire(&decoration)?;
                    }
                }
                let listener = decoration.state.marker_listener.borrow_mut().take();
                if let Some(listener) = listener {
                    listener.dispose();
                }
                Ok(())
            }));
        self.state
            .decorations
            .borrow_mut()
            .insert(decoration.clone())?;
        self.state.cache.add(decoration.clone())?;
        self.state.registered.fire(&decoration)?;
        Ok(Some(decoration))
    }
    pub fn reset(&self) -> Result<(), DecorationError> {
        let snapshot = self.decorations()?;
        for decoration in snapshot {
            decoration.dispose()?;
        }
        self.state.decorations.borrow_mut().clear();
        self.state.cache.clear();
        Ok(())
    }
    pub fn dispose(&self) -> Result<(), DecorationError> {
        if self.state.disposed.replace(true) {
            return Ok(());
        }
        // Upstream registers the cache and event emitters before the reset callback.
        self.state.cache.dispose();
        self.state.registered.dispose();
        self.state.removed.dispose();
        self.reset()?;
        let listener = self.state.activation_listener.borrow_mut().take();
        if let Some(listener) = listener {
            listener.dispose();
        }
        Ok(())
    }
    pub fn get_decorations_at_cell(
        &self,
        x: f64,
        line: impl Into<MarkerLine>,
        layer: Option<DecorationLayer>,
    ) -> DecorationCellIterator {
        self.state
            .cache
            .at_cell(x, line, layer)
            .retain_service(self.clone())
    }
    pub fn for_each_decoration_at_cell(
        &self,
        x: f64,
        line: impl Into<MarkerLine>,
        layer: Option<DecorationLayer>,
        mut callback: impl FnMut(Decoration),
    ) {
        for decoration in self.get_decorations_at_cell(x, line, layer) {
            callback(decoration);
        }
    }
}
