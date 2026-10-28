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
