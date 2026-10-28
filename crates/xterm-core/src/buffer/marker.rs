// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Ported from common/buffer/Marker.ts.

//! Shared marker identity and synchronous invalidation. Disposal is explicit and
//! fallible; dropping a marker handle never invokes user callbacks.

use crate::event::{EmitError, Emitter, Event};
use crate::lifecycle::{Disposable, DisposableHandle};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::sync::Mutex;

// Upstream's static counter is shared by every Marker, not by each Buffer.
// This fixed critical section avoids version-specific atomic update APIs and
// contention retry loops. It contains no allocations or user callbacks.
// JS increments cease to advance at 2^53. Preserve even that boundary behavior.
const JS_ID_LIMIT: u64 = 1 << 53;
static NEXT_ID: Mutex<u64> = Mutex::new(1);

fn following_id(id: u64) -> u64 {
    (id + 1).min(JS_ID_LIMIT)
}

struct State {
    id: u64,
    line: Cell<i64>,
    disposed: Cell<bool>,
    on_dispose: Emitter<()>,
    resources: RefCell<Vec<DisposableHandle>>,
}

/// A clone is another handle to the same marker, not a new marker with a new id.
#[derive(Clone)]
pub struct Marker {
    state: Rc<State>,
}

/// Capture a weak marker in callbacks to avoid caller-created ownership cycles.
#[derive(Clone)]
pub struct WeakMarker {
    state: Weak<State>,
}

impl WeakMarker {
    pub fn upgrade(&self) -> Option<Marker> {
        self.state.upgrade().map(|state| Marker { state })
    }
}

impl Marker {
    pub fn new(line: i64) -> Self {
        let id = {
            // No path while this guard is held can panic: the private counter
            // stays <=2^53 and only a bounded integer update executes. Thus the
            // lock cannot be poisoned; it is released before any allocation.
            let mut next = NEXT_ID
                .lock()
                .expect("marker id critical section cannot panic");
            let id = *next;
            *next = following_id(id);
            id
        };
        Self {
            state: Rc::new(State {
                id,
                line: Cell::new(line),
                disposed: Cell::new(false),
                on_dispose: Emitter::new(),
                resources: RefCell::new(Vec::new()),
            }),
        }
    }

    pub fn id(&self) -> u64 {
        self.state.id
    }

    /// Compare object identity, which remains distinct if numeric ids saturate.
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }

    pub fn line(&self) -> i64 {
        self.state.line.get()
    }

    /// Like the upstream public field, this remains writable after disposal.
    pub fn set_line(&self, line: i64) {
        self.state.line.set(line);
    }

    pub fn is_disposed(&self) -> bool {
        self.state.disposed.get()
    }

    pub fn on_dispose(&self) -> Event<()> {
        self.state.on_dispose.event()
    }

    pub fn downgrade(&self) -> WeakMarker {
        WeakMarker {
            state: Rc::downgrade(&self.state),
        }
    }

    /// Retain duplicates and late registrations, returning the exact same handle.
    /// Unlike DisposableStore, registration after disposal does not invoke it.
    pub fn register(&self, resource: DisposableHandle) -> DisposableHandle {
        self.state.resources.borrow_mut().push(Rc::clone(&resource));
        resource
    }

    /// Invalidate before firing, then dispose resources in live insertion order.
    ///
    /// A listener error or panic stops cleanup, with the marker already disposed.
    /// A later call does not retry or replay cleanup, matching JavaScript throws.
    /// This fallible method deliberately does not implement the infallible
    /// `Disposable` trait, since doing so would hide errors or introduce panics.
    pub fn dispose(&self) -> Result<(), EmitError> {
        if self.state.disposed.replace(true) {
            return Ok(());
        }
        self.state.line.set(-1);
        self.state.on_dispose.fire(&())?;
        // The source registers its emitter before user resources.
        self.state.on_dispose.dispose();
        let mut index = 0;
        loop {
            let resource = self.state.resources.borrow().get(index).cloned();
            let Some(resource) = resource else {
                break;
            };
            resource.dispose();
            index += 1;
        }
        let removed = self.state.resources.take();
        drop(removed);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{following_id, JS_ID_LIMIT};

    #[test]
    fn allocator_preserves_javascript_precision_limit() {
        assert_eq!(following_id(1), 2);
        assert_eq!(following_id(JS_ID_LIMIT - 1), JS_ID_LIMIT);
        assert_eq!(following_id(JS_ID_LIMIT), JS_ID_LIMIT);
    }
}
