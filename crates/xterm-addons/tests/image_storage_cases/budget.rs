use crate::support::*;
use xterm_addons::image::storage::*;

#[test]
fn fifo_limit_counts_original_and_distinct_resized_surfaces() {
    let f = Fixture::new(
        8,
        2,
        CellSize {
            width: 1000.0,
            height: 1000.0,
        },
    );
    f.storage.set_limit(0.5).unwrap();
    let mut ids = Vec::new();
    for x in 0..3 {
        f.set_cursor(x, 0);
        ids.push(
            f.storage
                .add_image(raster(200, 200), Default::default())
                .unwrap(),
        );
    }
    f.set_cursor(3, 0);
    let newest = f
        .storage
        .add_image(raster(100, 100), Default::default())
        .unwrap();
    assert!(f.storage.image_spec(ids[0]).is_none());
    assert!(f.storage.image_spec(ids[1]).is_some());
    assert!(f.storage.image_spec(ids[2]).is_some());
    assert_eq!(f.storage.get_usage().unwrap(), 0.36);
    f.storage
        .image_spec(ids[2])
        .unwrap()
        .replace_actual_resource(Some(raster(400, 400)));
    assert_eq!(f.storage.get_usage().unwrap(), 1.0);
    f.storage.set_limit(0.5).unwrap();
    assert!(f.storage.image_spec(ids[1]).is_none());
    assert!(f.storage.image_spec(ids[2]).is_none());
    assert!(f.storage.image_spec(newest).is_some());
    assert_eq!(f.storage.get_usage().unwrap(), 0.04);
    f.storage.set_limit(f64::NAN).unwrap();
    assert_eq!(f.storage.get_limit(), 0.0);
    assert_eq!(f.storage.image_count(), 0);
}

#[test]
fn preconverted_pixels_accept_full_range_and_commit_before_failed_eviction() {
    use std::rc::Rc;
    let f = Fixture::new(4, 2, CELL_SIZE_DEFAULT);
    f.storage.set_pixel_limit(u32::MAX).unwrap();
    assert_eq!(f.storage.get_limit(), f64::from(u32::MAX) / 250_000.0);
    let id = f
        .storage
        .add_image(raster(1, 1), Default::default())
        .unwrap();
    let storage = f.storage.clone();
    f.storage.set_on_image_deleted(Some(Rc::new(move |deleted| {
        assert_eq!(deleted, id);
        assert_eq!(storage.get_limit(), 0.0);
        Err(StorageError::InvalidLimit)
    })));
    let error = f.storage.set_pixel_limit(0).unwrap_err();
    let mut cause: &(dyn std::error::Error + 'static) = &error;
    loop {
        if matches!(
            cause.downcast_ref::<StorageError>(),
            Some(StorageError::InvalidLimit)
        ) {
            break;
        }
        cause = cause
            .source()
            .expect("eviction retains the callback's error");
    }
    assert_eq!(f.storage.get_limit(), 0.0);
    assert_eq!(f.storage.image_count(), 0);
    f.storage.set_on_image_deleted(None);
}

#[test]
fn preconverted_constructor_does_not_revalidate_or_log_pixel_budgets() {
    use std::{cell::Cell, rc::Rc};
    let f = Fixture::new(4, 2, CELL_SIZE_DEFAULT);
    for pixels in [0, 1, 2_500_000, u32::MAX] {
        let host = TerminalStorageHost::from_terminal(
            &f.terminal,
            Rc::new(|_, _| panic!("preconverted construction must not log diagnostics")),
        )
        .unwrap();
        let storage = ImageStorage::new_with_pixel_limit(
            Rc::new(host),
            f.renderer.clone(),
            Rc::new(Cell::new(false)),
            pixels,
        );
        assert_eq!(storage.get_limit(), f64::from(pixels) / 250_000.0);
        assert_eq!(storage.image_count(), 0);
        assert_eq!(storage.get_usage().unwrap(), 0.0);
    }
}

#[test]
fn invalid_limits_leave_budget_unchanged_and_nan_remains_source_accepted() {
    let f = Fixture::new(4, 2, CELL_SIZE_DEFAULT);
    for value in [0.499999, 1000.0001, f64::INFINITY, f64::NEG_INFINITY, -0.0] {
        assert!(matches!(
            f.storage.set_limit(value),
            Err(StorageError::InvalidLimit)
        ));
        assert_eq!(f.storage.get_limit(), 128.0);
    }
    f.storage.set_limit(1000.0).unwrap();
    assert_eq!(f.storage.get_limit(), 1000.0);
    f.storage.set_limit(0.5).unwrap();
    assert_eq!(f.storage.get_limit(), 0.5);
    f.storage.set_limit(f64::NAN).unwrap();
    let id = f
        .storage
        .add_image(raster(1, 1), Default::default())
        .unwrap();
    // Eviction precedes insertion; an oversize new image is not rejected here.
    assert_eq!(f.storage.get_limit(), 0.0);
    assert!(f.storage.image_spec(id).is_some());
    assert_eq!(f.storage.get_usage().unwrap(), 0.000004);
}
