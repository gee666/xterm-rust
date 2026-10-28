use crate::support::*;
use xterm_addons::image::storage::*;

#[test]
fn render_merges_consecutive_tiles_and_places_missing_image_placeholder_first() {
    let f = Fixture::new(
        6,
        2,
        CellSize {
            width: 1.0,
            height: 1.0,
        },
    );
    let a = f
        .storage
        .add_image(raster(2, 1), Default::default())
        .unwrap();
    f.set_cursor(3, 0);
    let b = f
        .storage
        .add_image(
            raster(2, 1),
            AddImageOptions {
                z_index: -1.0,
                ..Default::default()
            },
        )
        .unwrap();
    f.storage.delete_image(a).unwrap();
    f.events.borrow_mut().clear();
    f.storage.render(0, 0).unwrap();
    assert_eq!(
        f.events(),
        [
            "insert:Top",
            "rescale",
            "lines:0:0",
            "placeholder:0:0:2",
            &format!("draw:{b}:0:3:0:2")
        ]
    );
    f.storage.delete_image(b).unwrap();
    f.events.borrow_mut().clear();
    f.storage.render(0, 0).unwrap();
    f.storage.render(0, 0).unwrap();
    assert_eq!(
        f.events(),
        ["rescale", "clear:None", "remove:Top", "rescale"]
    );
}

#[test]
fn z_order_is_stable_and_top_insertion_failure_aborts_before_rescale() {
    let f = Fixture::new(
        6,
        2,
        CellSize {
            width: 1.0,
            height: 1.0,
        },
    );
    let a = f
        .storage
        .add_image(
            raster(1, 1),
            AddImageOptions {
                z_index: 5.0,
                ..Default::default()
            },
        )
        .unwrap();
    f.set_cursor(2, 0);
    let b = f
        .storage
        .add_image(
            raster(1, 1),
            AddImageOptions {
                layer: ImageLayer::Bottom,
                z_index: -4.0,
                ..Default::default()
            },
        )
        .unwrap();
    f.renderer.deny_top.set(true);
    f.storage.render(0, 0).unwrap();
    assert_eq!(f.events(), ["insert:Top"]);
    f.events.borrow_mut().clear();
    f.renderer.deny_top.set(false);
    f.storage.render(0, 0).unwrap();
    assert_eq!(
        f.events(),
        [
            "insert:Top",
            "insert:Bottom",
            "rescale",
            "lines:0:0",
            &format!("draw:{b}:0:2:0:1"),
            &format!("draw:{a}:0:0:0:1")
        ]
    );
}

#[test]
fn extraction_delegates_exact_tile_and_missing_indices_return_absence() {
    let f = Fixture::new(
        4,
        2,
        CellSize {
            width: 1.0,
            height: 1.0,
        },
    );
    let id = f
        .storage
        .add_image(raster(2, 1), Default::default())
        .unwrap();
    let result = f
        .storage
        .extract_tile_at_buffer_cell(1, 0)
        .unwrap()
        .unwrap();
    assert!(result.same_identity(f.renderer.extraction.borrow().as_ref().unwrap()));
    assert_eq!(f.events(), [format!("extract:{id}:1")]);
    assert!(f
        .storage
        .extract_tile_at_buffer_cell(2, 0)
        .unwrap()
        .is_none());
    assert!(f
        .storage
        .get_image_at_buffer_cell_number(f64::NAN, 0.0)
        .unwrap()
        .is_none());
    assert!(f
        .storage
        .get_image_at_buffer_cell_number(0.0, f64::NAN)
        .unwrap()
        .is_none());
    assert!(f
        .storage
        .extract_tile_at_buffer_cell_number(-1.0, 0.0)
        .unwrap()
        .is_none());
    f.tile(0, 0).unwrap().tile_id.set(-1.0);
    assert!(f
        .storage
        .extract_tile_at_buffer_cell(0, 0)
        .unwrap()
        .is_none());
}
