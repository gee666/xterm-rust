use crate::support::*;
use std::rc::Rc;
use xterm_addons::image::storage::CELL_SIZE_DEFAULT;

#[test]
fn live_entries_follow_resets_appends_identity_and_permanent_exhaustion() {
    let f = Fixture::new(20, 10, CELL_SIZE_DEFAULT);
    let first = f
        .storage
        .add_image(raster(1, 1), Default::default())
        .unwrap();
    let mut entries = f.storage.image_entries();
    let (id, spec) = entries.next().unwrap();
    assert_eq!(id, first);
    assert!(Rc::ptr_eq(&spec, &f.storage.image_spec(id).unwrap()));
    f.storage.reset().unwrap();
    let second = f
        .storage
        .add_image(raster(1, 1), Default::default())
        .unwrap();
    assert_eq!(entries.next().unwrap().0, second);
    assert!(entries.next().is_none());
    f.set_cursor(1, 0);
    f.storage
        .add_image(raster(1, 1), Default::default())
        .unwrap();
    assert!(entries.next().is_none());
    assert_eq!(f.storage.image_entries().count(), 2);
}

#[test]
fn advancing_entries_does_not_borrow_the_store_during_consumer_callbacks() {
    let f = Fixture::new(20, 10, CELL_SIZE_DEFAULT);
    for x in 0..3 {
        f.set_cursor(x, 0);
        f.storage
            .add_image(raster(1, 1), Default::default())
            .unwrap();
    }
    let mut seen = Vec::new();
    for (id, _) in f.storage.image_entries() {
        seen.push(id);
        f.storage.delete_image(id).unwrap();
        if seen.len() == 1 {
            f.set_cursor(3, 0);
            f.storage
                .add_image(raster(1, 1), Default::default())
                .unwrap();
        }
    }
    assert_eq!(seen, [1, 2, 3, 4]);
    assert_eq!(f.storage.image_count(), 0);
}
