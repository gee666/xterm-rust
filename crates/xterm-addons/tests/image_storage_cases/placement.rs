use crate::support::*;
use std::{cell::Cell, rc::Rc};
use xterm_addons::image::storage::*;
use xterm_core::{
    buffer::{constants::bg_flags, AttributeData},
    event::EmitError,
};

#[test]
fn actual_terminal_tiles_preserve_text_attributes_and_iip_cursor() {
    let f = Fixture::new(
        6,
        3,
        CellSize {
            width: 2.0,
            height: 2.0,
        },
    );
    f.set_cursor(1, 1);
    let buffer = f
        .terminal
        .with_state(|state| state.active_buffer())
        .unwrap();
    let line = buffer.borrow().lines.get(1).unwrap();
    line.borrow_mut().set_cell_from_codepoint(
        1,
        0x41,
        1,
        &AttributeData {
            fg: 0x010203,
            bg: 0x040506,
            ..Default::default()
        },
    );
    let image = raster(5, 3);
    let id = f
        .storage
        .add_image(
            image.clone(),
            AddImageOptions {
                cursor_position: CursorPosition::Iip,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(f.cursor(), (4, 2, 0));
    assert_eq!(f.storage.image_spec(id).unwrap().tile_count.get(), 6.0);
    for (row, first) in [(1, 0.0), (2, 3.0)] {
        for col in 1..4 {
            let tile = f.tile(col, row).unwrap();
            assert_eq!(tile.image_id.get(), id as i64);
            assert_eq!(tile.tile_id.get(), first + (col - 1) as f64);
        }
    }
    let cell = line.borrow().get_cell(1).unwrap();
    assert_eq!(cell.get_code(), Some(0x41));
    assert_eq!(cell.fg, 0x010203);
    assert_eq!(cell.bg & !bg_flags::HAS_EXTENDED, 0x040506);
    assert!(f
        .storage
        .get_image_at_buffer_cell(1, 1)
        .unwrap()
        .unwrap()
        .same_identity(&image));
    assert_eq!(f.storage.get_usage().unwrap(), 0.00006);
    assert!(f.storage.image_spec(id).unwrap().marker.is_some());
}

#[test]
fn non_scrolling_clips_right_and_restores_origin_after_bottom_break() {
    let f = Fixture::new(
        4,
        2,
        CellSize {
            width: 2.0,
            height: 2.0,
        },
    );
    f.set_cursor(3, 1);
    let id = f
        .storage
        .add_image(
            raster(12, 8),
            AddImageOptions {
                scrolling: false,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(f.cursor(), (3, 1, 0));
    assert_eq!(f.storage.image_spec(id).unwrap().tile_count.get(), 8.0);
    for col in 0..4 {
        assert_eq!(f.tile(col, 0).unwrap().tile_id.get(), col as f64);
        assert_eq!(f.tile(col, 1).unwrap().tile_id.get(), 6.0 + col as f64);
    }
}

#[test]
fn overwrites_reuse_isolated_payload_and_delete_zero_tile_images() {
    let f = Fixture::new(
        4,
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
    let first = f.tile(0, 0).unwrap();
    let b = f
        .storage
        .add_image(raster(1, 1), Default::default())
        .unwrap();
    assert!(Rc::ptr_eq(&first, &f.tile(0, 0).unwrap()));
    assert_eq!(f.storage.image_spec(a).unwrap().tile_count.get(), 1.0);
    assert_eq!(f.storage.image_spec(b).unwrap().tile_count.get(), 1.0);
    f.set_cursor(1, 0);
    f.storage
        .add_image(raster(1, 1), Default::default())
        .unwrap();
    assert!(f.storage.image_spec(a).is_none());
    assert_eq!(f.storage.image_count(), 2);
}

#[test]
fn linefeed_resize_uses_captured_width_without_rejecting_out_of_view_stores() {
    let f = Fixture::new(
        6,
        3,
        CellSize {
            width: 1.0,
            height: 1.0,
        },
    );
    f.set_cursor(4, 0);
    let weak = Rc::downgrade(&f.terminal);
    let resized = Rc::new(Cell::new(false));
    let once = resized.clone();
    let listener = f.terminal.on_line_feed().try_subscribe(move |_| {
        if !once.replace(true) {
            weak.upgrade()
                .unwrap()
                .resize(2, 1)
                .map_err(EmitError::callback)?;
        }
        Ok(())
    });
    let id = f
        .storage
        .add_image(raster(1, 2), Default::default())
        .unwrap();
    listener.dispose();
    assert!(resized.get());
    assert_eq!(f.storage.image_spec(id).unwrap().tile_count.get(), 2.0);
    assert_eq!(f.cursor().0, 4);
}

#[test]
fn fallback_cell_size_and_empty_images_keep_source_metadata() {
    let f = Fixture::new(
        5,
        3,
        CellSize {
            width: -1.0,
            height: 9.0,
        },
    );
    let id = f
        .storage
        .add_image(raster(8, 15), Default::default())
        .unwrap();
    assert_eq!(
        f.storage.image_spec(id).unwrap().original_cell_size,
        CELL_SIZE_DEFAULT
    );
    assert_eq!(f.storage.image_spec(id).unwrap().tile_count.get(), 4.0);
    let empty = f
        .storage
        .add_image(raster(0, 0), Default::default())
        .unwrap();
    assert_eq!(f.storage.image_spec(empty).unwrap().tile_count.get(), 0.0);
    f.storage
        .add_image(raster(1, 1), Default::default())
        .unwrap();
    assert!(f.storage.image_spec(empty).is_none());
}

#[test]
fn width_growth_preserves_the_source_empty_content_check_quirk() {
    let f = Fixture::new(
        3,
        2,
        CellSize {
            width: 1.0,
            height: 1.0,
        },
    );
    let id = f
        .storage
        .add_image(raster(6, 1), Default::default())
        .unwrap();
    f.terminal.resize(6, 2).unwrap();
    let buffer = f
        .terminal
        .with_state(|state| state.active_buffer())
        .unwrap();
    let line = buffer.borrow().lines.get(0).unwrap();
    line.borrow_mut()
        .set_cell_from_codepoint(4, 0x58, 1, &AttributeData::default());
    f.storage.viewport_resize(6, 2).unwrap();
    for col in 0..6 {
        assert_eq!(f.tile(col, 0).unwrap().tile_id.get(), col as f64);
    }
    assert_eq!(line.borrow().get_code_point(4), Some(0x58));
    assert_eq!(f.storage.image_spec(id).unwrap().tile_count.get(), 6.0);
}
