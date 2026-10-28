// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};
use xterm_addons::image::encoded::EncodedBytes;
use xterm_addons::image::kitty::storage::*;
use xterm_addons::image::storage::{AddImageOptions, ImageLayer, ImageResource, StorageError};
use xterm_addons::image::types::RgbaImage;

#[derive(Default)]
struct Host {
    next: Cell<u64>,
    images: RefCell<BTreeMap<u64, ImageResource>>,
    callback: RefCell<Option<DeletedCallback>>,
    events: RefCell<Vec<String>>,
}
impl KittyPlacementStorage for Host {
    fn add_image(&self, image: ImageResource, _: AddImageOptions) -> Result<u64, StorageError> {
        let id = self.next.get() + 1;
        self.next.set(id);
        self.images.borrow_mut().insert(id, image);
        self.events.borrow_mut().push(format!("add:{id}"));
        Ok(id)
    }
    fn delete_image(&self, id: u64) -> Result<(), StorageError> {
        let removed = self.images.borrow_mut().remove(&id);
        drop(removed);
        self.events.borrow_mut().push(format!("delete:{id}"));
        let callback = self.callback.borrow().clone();
        if let Some(callback) = callback {
            callback(id)?;
        }
        Ok(())
    }
    fn on_deleted(&self) -> Option<DeletedCallback> {
        self.callback.borrow().clone()
    }
    fn set_on_deleted(&self, callback: Option<DeletedCallback>) {
        *self.callback.borrow_mut() = callback;
    }
}
fn bytes(value: u8) -> KittyImageBytes {
    KittyImageBytes {
        data: EncodedBytes::from_owned(vec![value]),
        width: 1.0,
        height: 1.0,
        format: 32.0,
        compression: Vec::new(),
    }
}
fn surface() -> ImageResource {
    ImageResource::from_pixels(RgbaImage::new(1, 1, vec![0, 0, 0, 255]).unwrap())
}
fn place(storage: &KittyImageStorage, id: f64) -> u64 {
    storage
        .add_image(id, surface(), true, ImageLayer::Top, 0.0)
        .unwrap()
}

#[test]
fn explicit_ids_do_not_advance_automatic_ids_and_numeric_keys_use_same_value_zero() {
    let storage = KittyImageStorage::new(Rc::new(Host::default()));
    storage.store_image(Some(0.0), bytes(1)).unwrap();
    storage.store_image(Some(1.0), bytes(2)).unwrap();
    assert_eq!(storage.last_image_id().unwrap(), 0.0);
    assert_eq!(storage.store_image(None, bytes(3)).unwrap(), 1.0);
    assert_eq!(
        storage
            .get_image(1.0)
            .unwrap()
            .unwrap()
            .bytes
            .borrow()
            .data
            .as_memory(),
        Some(&[3][..])
    );
    storage.store_image(Some(-0.0), bytes(4)).unwrap();
    storage.store_image(Some(f64::NAN), bytes(5)).unwrap();
    assert_eq!(
        storage.get_image(0.0).unwrap().unwrap().id.get().to_bits(),
        (-0.0_f64).to_bits()
    );
    assert!(storage.get_image(f64::NAN).unwrap().is_some());
    assert_eq!(storage.images().len().unwrap(), 3);
}
#[test]
fn max256_evicts_only_undisplayed_entries_down_to128_before_inserting() {
    let host = Rc::new(Host::default());
    let storage = KittyImageStorage::new(host);
    for id in 1..=256 {
        storage.store_image(Some(f64::from(id)), bytes(0)).unwrap();
    }
    place(&storage, 1.0);
    storage.store_image(Some(257.0), bytes(0)).unwrap();
    let keys: Vec<_> = storage
        .images()
        .iter()
        .map(|entry| entry.unwrap().0)
        .collect();
    assert_eq!(keys.len(), 129);
    assert_eq!(&keys[..2], &[1.0, 130.0]);
    assert_eq!(keys.last(), Some(&257.0));
}
#[test]
fn stale_old_placement_eviction_does_not_remove_current_source_data() {
    let host = Rc::new(Host::default());
    let storage = KittyImageStorage::new(host.clone());
    storage.store_image(Some(1.0), bytes(1)).unwrap();
    let old = place(&storage, 1.0);
    let current = place(&storage, 1.0);
    host.delete_image(old).unwrap();
    assert!(storage.get_image(1.0).unwrap().is_some());
    assert_eq!(storage.placements().get(1.0).unwrap(), Some(current));
    host.delete_image(current).unwrap();
    assert!(storage.get_image(1.0).unwrap().is_none());
}
#[test]
fn prior_delete_callback_observes_old_data_before_wrapper_updates_and_overwrite() {
    let host = Rc::new(Host::default());
    let owner = Rc::new(RefCell::new(Weak::<KittyImageStorage>::new()));
    let observed = Rc::new(RefCell::new(Vec::new()));
    let weak_owner = Rc::clone(&owner);
    let output = Rc::clone(&observed);
    host.set_on_deleted(Some(Rc::new(move |_| {
        let store = weak_owner.borrow().upgrade().unwrap();
        let image = store.get_image(1.0)?.unwrap();
        output
            .borrow_mut()
            .push(image.bytes.borrow().data.as_memory().unwrap()[0]);
        Ok(())
    })));
    let storage = Rc::new(KittyImageStorage::new(host));
    *owner.borrow_mut() = Rc::downgrade(&storage);
    storage.store_image(Some(1.0), bytes(1)).unwrap();
    place(&storage, 1.0);
    storage.store_image(Some(1.0), bytes(2)).unwrap();
    assert_eq!(*observed.borrow(), [1]);
    assert_eq!(
        storage
            .get_image(1.0)
            .unwrap()
            .unwrap()
            .bytes
            .borrow()
            .data
            .as_memory(),
        Some(&[2][..])
    );
}
#[test]
fn delete_all_observes_callback_appends_without_holding_map_borrows() {
    let host = Rc::new(Host::default());
    let owner = Rc::new(RefCell::new(Weak::<KittyImageStorage>::new()));
    let once = Rc::new(Cell::new(false));
    let weak_owner = Rc::clone(&owner);
    host.set_on_deleted(Some(Rc::new(move |_| {
        if !once.replace(true) {
            let storage = weak_owner.borrow().upgrade().unwrap();
            storage.store_image(Some(2.0), bytes(2))?;
            place(&storage, 2.0);
        }
        Ok(())
    })));
    let storage = Rc::new(KittyImageStorage::new(host.clone()));
    *owner.borrow_mut() = Rc::downgrade(&storage);
    storage.store_image(Some(1.0), bytes(1)).unwrap();
    place(&storage, 1.0);
    storage.delete_all().unwrap();
    assert_eq!(
        *host.events.borrow(),
        ["add:1", "delete:1", "add:2", "delete:2"]
    );
    assert!(host.images.borrow().is_empty());
    assert!(storage.images().is_empty().unwrap());
    assert!(storage.placements().is_empty().unwrap());
}
#[test]
fn retained_views_iterators_and_record_identity_follow_source_map_mutation() {
    let storage = KittyImageStorage::new(Rc::new(Host::default()));
    let view = storage.images();
    storage.store_image(Some(1.0), bytes(1)).unwrap();
    storage.store_image(Some(2.0), bytes(2)).unwrap();
    let mut iterator = view.iter();
    let first = iterator.next().unwrap().unwrap();
    assert_eq!(first.0, 1.0);
    first.1.id.set(99.0);
    assert_eq!(storage.get_image(1.0).unwrap().unwrap().id.get(), 99.0);
    assert!(Rc::ptr_eq(
        &first.1,
        &storage.get_image(1.0).unwrap().unwrap()
    ));
    storage.reset().unwrap();
    storage.store_image(Some(3.0), bytes(3)).unwrap();
    assert_eq!(iterator.next().unwrap().unwrap().0, 3.0);
    assert!(iterator.next().is_none());
    storage.store_image(Some(4.0), bytes(4)).unwrap();
    assert!(iterator.next().is_none());
    assert!(view.same_identity(&storage.images()));
    assert_eq!(view.len().unwrap(), 2);
}
#[test]
fn dispose_restores_callback_only_if_it_still_owns_the_slot() {
    let host = Rc::new(Host::default());
    let previous: DeletedCallback = Rc::new(|_| Ok(()));
    host.set_on_deleted(Some(Rc::clone(&previous)));
    let storage = KittyImageStorage::new(host.clone());
    storage.dispose().unwrap();
    assert!(Rc::ptr_eq(&host.on_deleted().unwrap(), &previous));
    let storage = KittyImageStorage::new(host.clone());
    let replacement: DeletedCallback = Rc::new(|_| Ok(()));
    host.set_on_deleted(Some(Rc::clone(&replacement)));
    storage.dispose().unwrap();
    assert!(Rc::ptr_eq(&host.on_deleted().unwrap(), &replacement));
}
