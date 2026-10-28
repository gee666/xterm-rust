use crate::support::*;
use std::{cell::Cell, error::Error, rc::Rc};
use xterm_addons::image::storage::*;
use xterm_core::buffer::AttributeData;

#[test]
fn bitmap_conversion_and_deletion_preserve_identity_close_order_and_single_callback() {
    let f = Fixture::new(
        4,
        2,
        CellSize {
            width: 1.0,
            height: 1.0,
        },
    );
    let original = bitmap("original", 1, 1, f.events.clone(), None);
    let id = f
        .storage
        .add_image(original.clone(), Default::default())
        .unwrap();
    let canvas = f.storage.get_image_at_buffer_cell(0, 0).unwrap().unwrap();
    assert!(!canvas.same_identity(&original));
    assert_eq!(canvas.pixels().unwrap().pixels(), [0, 0, 0, 0]);
    let weak = f.storage.downgrade();
    let events = f.events.clone();
    f.storage.set_on_image_deleted(Some(Rc::new(move |id| {
        let storage = weak.upgrade().unwrap();
        assert!(storage.image_spec(id).is_none());
        events.borrow_mut().push(format!("deleted:{id}"));
        Ok(())
    })));
    f.events.borrow_mut().clear();
    f.storage.delete_image(id).unwrap();
    f.storage.delete_image(id).unwrap();
    assert_eq!(f.events(), ["close:original", "deleted:1"]);
    assert_eq!(original.width().unwrap(), 0);
    assert_eq!(original.height().unwrap(), 0);
}

#[test]
fn close_failure_removes_mapping_before_error_and_preserves_cause_identity() {
    let f = Fixture::new(
        4,
        2,
        CellSize {
            width: 1.0,
            height: 1.0,
        },
    );
    let original = Rc::new(Failure("bitmap close failed"));
    let cause: Rc<dyn Error> = original.clone();
    let image = bitmap("failing", 1, 1, f.events.clone(), Some(cause));
    let id = f.storage.add_image(image, Default::default()).unwrap();
    let called = Rc::new(Cell::new(false));
    let flag = called.clone();
    f.storage.set_on_image_deleted(Some(Rc::new(move |_| {
        flag.set(true);
        Ok(())
    })));
    let error = f.storage.delete_image(id).unwrap_err();
    assert!(f.storage.image_spec(id).is_none());
    assert!(!called.get());
    let mut current: &(dyn Error + 'static) = &error;
    loop {
        if let Some(found) = current.downcast_ref::<Failure>() {
            assert!(std::ptr::eq(found, original.as_ref()));
            break;
        }
        current = current.source().expect("original callback error retained");
    }
}

#[test]
fn added_callback_failure_leaves_inserted_tiles_and_entry_committed() {
    let f = Fixture::new(
        4,
        2,
        CellSize {
            width: 1.0,
            height: 1.0,
        },
    );
    f.storage.set_on_image_added(Some(Rc::new(|| {
        Err(StorageError::host(Failure("added callback")))
    })));
    let error = f
        .storage
        .add_image(raster(1, 1), Default::default())
        .unwrap_err();
    assert_eq!(error.to_string(), "added callback");
    assert_eq!(f.storage.image_count(), 1);
    assert_eq!(f.tile(0, 0).unwrap().image_id.get(), 1);
    f.storage.set_on_image_added(None);
    f.storage.reset().unwrap();
    assert_eq!(f.storage.image_count(), 0);
}

#[test]
fn alternate_wipe_preserves_normal_images_and_leaves_orphan_tiles_for_placeholders() {
    let f = Fixture::new(
        4,
        2,
        CellSize {
            width: 1.0,
            height: 1.0,
        },
    );
    let normal = f
        .storage
        .add_image(raster(1, 1), Default::default())
        .unwrap();
    let service = f
        .terminal
        .with_state(|state| state.buffer_service.clone())
        .unwrap();
    service
        .buffers
        .activate_alt_buffer(Some(&AttributeData::default()))
        .unwrap();
    let a = f
        .storage
        .add_image(raster(1, 1), Default::default())
        .unwrap();
    f.set_cursor(1, 0);
    let b = f
        .storage
        .add_image(raster(1, 1), Default::default())
        .unwrap();
    f.storage.wipe_alternate().unwrap();
    assert!(f.storage.image_spec(normal).is_some());
    assert!(f.storage.image_spec(a).is_none());
    assert!(f.storage.image_spec(b).is_none());
    assert_eq!(f.tile(0, 0).unwrap().image_id.get(), a as i64);
    assert!(f.storage.get_image_at_buffer_cell(0, 0).unwrap().is_none());
}
