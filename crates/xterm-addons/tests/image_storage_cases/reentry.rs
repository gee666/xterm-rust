use crate::support::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_addons::image::storage::*;
use xterm_core::event::EmitError;

#[test]
fn reset_observes_images_appended_by_deleted_callbacks_and_keeps_id_counter() {
    let f = Fixture::new(
        6,
        2,
        CellSize {
            width: 1.0,
            height: 1.0,
        },
    );
    f.storage
        .add_image(raster(1, 1), Default::default())
        .unwrap();
    let ids = Rc::new(RefCell::new(Vec::new()));
    let observed = ids.clone();
    let weak = f.storage.downgrade();
    f.storage.set_on_image_deleted(Some(Rc::new(move |id| {
        observed.borrow_mut().push(id);
        if id == 1 {
            weak.upgrade()
                .unwrap()
                .add_image(raster(1, 1), Default::default())?;
        }
        Ok(())
    })));
    f.storage.reset().unwrap();
    assert_eq!(*ids.borrow(), [1, 2]);
    assert_eq!(f.storage.image_count(), 0);
    f.storage.set_on_image_deleted(None);
    assert_eq!(
        f.storage
            .add_image(raster(1, 1), Default::default())
            .unwrap(),
        3
    );
}

#[test]
fn linefeed_nested_placement_uses_map_insertion_order_not_numeric_ids() {
    let f = Fixture::new(
        8,
        3,
        CellSize {
            width: 1000.0,
            height: 1000.0,
        },
    );
    let weak_terminal = Rc::downgrade(&f.terminal);
    let weak = f.storage.downgrade();
    let once = Cell::new(false);
    let nested = bitmap("inner", 200, 100, f.events.clone(), None);
    let listener = f.terminal.on_line_feed().try_subscribe(move |_| {
        if !once.replace(true) {
            set_cursor(&weak_terminal.upgrade().unwrap(), 3, 1);
            weak.upgrade()
                .unwrap()
                .add_image(nested.clone(), Default::default())
                .map_err(EmitError::callback)?;
        }
        Ok(())
    });
    let outer = f
        .storage
        .add_image(
            bitmap("outer", 200, 2000, f.events.clone(), None),
            Default::default(),
        )
        .unwrap();
    listener.dispose();
    assert_eq!(outer, 1);
    assert_eq!(f.storage.image_count(), 2);
    let deleted = Rc::new(RefCell::new(Vec::new()));
    let observed = deleted.clone();
    f.storage.set_on_image_deleted(Some(Rc::new(move |id| {
        observed.borrow_mut().push(id);
        Ok(())
    })));
    // Numeric eviction order is separate from insertion order: id1 is evicted
    // first even though the LF callback inserted id2 before id1.
    f.storage.set_limit(0.5).unwrap();
    assert_eq!(*deleted.borrow(), [1]);
    assert!(f.storage.image_spec(2).is_some());
    f.storage.reset().unwrap();
    assert_eq!(*deleted.borrow(), [1, 2]);
}

#[test]
fn reset_order_follows_nested_insertion_order_two_then_one() {
    let f = Fixture::new(
        8,
        3,
        CellSize {
            width: 1.0,
            height: 1.0,
        },
    );
    let terminal = Rc::downgrade(&f.terminal);
    let weak = f.storage.downgrade();
    let once = Cell::new(false);
    let listener = f.terminal.on_line_feed().try_subscribe(move |_| {
        if !once.replace(true) {
            set_cursor(&terminal.upgrade().unwrap(), 3, 1);
            weak.upgrade()
                .unwrap()
                .add_image(raster(1, 1), Default::default())
                .map_err(EmitError::callback)?;
        }
        Ok(())
    });
    f.storage
        .add_image(raster(1, 2), Default::default())
        .unwrap();
    listener.dispose();
    let deleted = Rc::new(RefCell::new(Vec::new()));
    let observed = deleted.clone();
    f.storage.set_on_image_deleted(Some(Rc::new(move |id| {
        observed.borrow_mut().push(id);
        Ok(())
    })));
    f.storage.reset().unwrap();
    assert_eq!(*deleted.borrow(), [2, 1]);
}

struct DropSlot {
    storage: WeakImageStorage,
    deleted: bool,
    count: Rc<Cell<usize>>,
}
impl Drop for DropSlot {
    fn drop(&mut self) {
        self.count.set(self.count.get() + 1);
        if let Some(storage) = self.storage.upgrade() {
            if self.deleted {
                storage.set_on_image_deleted(None);
            } else {
                storage.set_on_image_added(None);
            }
        }
    }
}
#[test]
fn replacing_callback_drops_old_capture_without_a_slot_borrow() {
    let f = Fixture::new(4, 2, CELL_SIZE_DEFAULT);
    let count = Rc::new(Cell::new(0));
    let guard = DropSlot {
        storage: f.storage.downgrade(),
        deleted: false,
        count: count.clone(),
    };
    f.storage.set_on_image_added(Some(Rc::new(move || {
        let _ = &guard;
        Ok(())
    })));
    f.storage.set_on_image_added(Some(Rc::new(|| Ok(()))));
    assert_eq!(count.get(), 1);
    assert!(f.storage.on_image_added().is_none());
    let guard = DropSlot {
        storage: f.storage.downgrade(),
        deleted: true,
        count: count.clone(),
    };
    f.storage.set_on_image_deleted(Some(Rc::new(move |_| {
        let _ = &guard;
        Ok(())
    })));
    f.storage.set_on_image_deleted(Some(Rc::new(|_| Ok(()))));
    assert_eq!(count.get(), 2);
    assert!(f.storage.on_image_deleted().is_none());
}
