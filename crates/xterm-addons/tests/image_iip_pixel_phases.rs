// Copyright (c) 2023 The xterm.js authors. MIT license.
use xterm_addons::image::{
    iip::{
        geometry::IipGeometry,
        handler::IipHandler,
        types::{
            EndPreparation, IipEffect, IipError, IipOptions, PixelEndPreparation,
            PixelResizePreparation,
        },
    },
    NameDecodePolicy,
};
const PNG: &str = "iVBORw0KGgoAAAAASUhEUgAAAAIAAAABAAAAAAAAAAA=";
fn handler() -> IipHandler {
    IipHandler::new(IipOptions::default(), NameDecodePolicy::NodeBuffer).unwrap()
}
fn put(handler: &mut IipHandler, text: &str) {
    let data: Vec<_> = text.chars().map(u32::from).collect();
    handler.put(&data, 0, data.len()).unwrap();
}
fn start(handler: &mut IipHandler) {
    handler.start();
    put(handler, &format!("File=inline=1;width=1:{PNG}"));
}
fn geometry() -> IipGeometry {
    IipGeometry {
        columns: 80.0,
        rows: 24.0,
        render: None,
        device_pixel_ratio: None,
    }
}
#[test]
fn pixel_tokens_capture_dimensions_but_read_live_payload_after_callbacks() {
    let mut handler = handler();
    start(&mut handler);
    let PixelEndPreparation::Check(check) = handler.prepare_end_pixels(true).unwrap() else {
        panic!("missing original comparison")
    };
    assert_eq!(check.area(), 2.0);
    let retained = handler.loaded_bytes();
    assert!(retained > 0);
    handler.reset();
    assert_eq!(handler.loaded_bytes(), 0);
    let EndPreparation::Geometry(token) = handler.finish_original_pixels(check, true).unwrap()
    else {
        panic!("missing resize")
    };
    let PixelResizePreparation::Check(check) =
        handler.prepare_resize_pixels(token, &geometry()).unwrap()
    else {
        panic!("missing resized comparison")
    };
    assert_eq!(check.area(), 21.0);
    // A nested transfer replaces the payload after the geometry locals exist.
    handler.start();
    put(&mut handler, "File=inline=1:AAAA");
    let current = handler.loaded_bytes();
    assert!(current > 0);
    let effects = handler.finish_resized_pixels(check, true).unwrap();
    let [IipEffect::Display(request)] = effects.as_slice() else {
        panic!("missing display")
    };
    // Source data8 stays empty until the nested transfer calls end. The
    // retained outer continuation must not finalize that nested decoder.
    assert!(request.bytes.is_empty());
    assert_eq!(handler.loaded_bytes(), current);
    assert_eq!((request.source.width, request.source.height), (2, 1));
    assert_eq!((request.target.width, request.target.height), (7.0, 3.0));
}
#[test]
fn rejected_original_keeps_payload_until_warning_but_second_rejection_releases_live_payload() {
    let mut handler = handler();
    start(&mut handler);
    let PixelEndPreparation::Check(check) = handler.prepare_end_pixels(true).unwrap() else {
        panic!()
    };
    let retained = handler.loaded_bytes();
    let EndPreparation::Complete(effects) = handler.finish_original_pixels(check, false).unwrap()
    else {
        panic!()
    };
    assert!(
        matches!(&effects[..],[IipEffect::Warning(text)] if text=="IIP: image dimension issue 2x1")
    );
    assert_eq!(handler.loaded_bytes(), retained);
    start(&mut handler);
    let PixelEndPreparation::Check(check) = handler.prepare_end_pixels(true).unwrap() else {
        panic!()
    };
    let EndPreparation::Geometry(token) = handler.finish_original_pixels(check, true).unwrap()
    else {
        panic!()
    };
    let PixelResizePreparation::Check(check) =
        handler.prepare_resize_pixels(token, &geometry()).unwrap()
    else {
        panic!()
    };
    handler.start();
    put(&mut handler, "File=inline=1:AAAA");
    assert!(handler.loaded_bytes() > 0);
    assert!(handler
        .finish_resized_pixels(check, false)
        .unwrap()
        .is_empty());
    assert_eq!(handler.loaded_bytes(), 0);
}
#[test]
fn pixel_tokens_reject_foreign_controllers_before_mutating_payload() {
    let mut first = handler();
    let mut second = handler();
    start(&mut first);
    start(&mut second);
    let PixelEndPreparation::Check(check) = first.prepare_end_pixels(true).unwrap() else {
        panic!()
    };
    assert!(matches!(
        second.finish_original_pixels(check, true),
        Err(IipError::ForeignEndToken)
    ));
    let PixelEndPreparation::Check(check) = first.prepare_end_pixels(true).unwrap() else {
        panic!()
    };
    let EndPreparation::Geometry(token) = first.finish_original_pixels(check, true).unwrap() else {
        panic!()
    };
    let PixelResizePreparation::Check(check) =
        first.prepare_resize_pixels(token, &geometry()).unwrap()
    else {
        panic!()
    };
    let retained = second.loaded_bytes();
    assert!(matches!(
        second.finish_resized_pixels(check, false),
        Err(IipError::ForeignEndToken)
    ));
    assert_eq!(second.loaded_bytes(), retained);
}
