use std::cell::RefCell;
use xterm_addons::image::{
    iip::{
        geometry::IipGeometry,
        handler::IipHandler,
        types::{IipEffect, IipOptions},
    },
    NameDecodePolicy,
};

const PNG: &str = "iVBORw0KGgoAAAAASUhEUgAAAAIAAAABAAAAAAAAAAA=";

fn geometry() -> IipGeometry {
    IipGeometry {
        columns: 80.0,
        rows: 24.0,
        render: None,
        device_pixel_ratio: None,
    }
}
fn write(handler: &RefCell<IipHandler>, text: &str) -> Vec<IipEffect> {
    let data: Vec<_> = text.chars().map(u32::from).collect();
    let mut handler = handler.borrow_mut();
    handler.start();
    handler.put(&data, 0, data.len()).unwrap();
    handler.end(true, &geometry()).unwrap()
}
fn handler() -> RefCell<IipHandler> {
    RefCell::new(
        IipHandler::new(
            IipOptions {
                pixel_limit: 2.0,
                ..IipOptions::default()
            },
            NameDecodePolicy::BrowserAtob,
        )
        .unwrap(),
    )
}

#[test]
fn diagnostic_payload_survives_until_successful_host_warning() {
    for (payload, message) in [
        ("A", "IIP: error during BASE64 decoding"),
        ("AAAA", "IIP: unsupported image type"),
        (PNG, "IIP: image dimension issue 2x1"),
    ] {
        let handler = handler();
        let effects = write(&handler, &format!("File=inline=1:{payload}"));
        let [IipEffect::Warning(warning)] = effects.as_slice() else {
            panic!("{effects:?}")
        };
        assert_eq!(warning, message);
        assert_eq!(handler.borrow().loaded_bytes(), payload.len());
        let failure = String::from("host warning failed");
        let result = IipHandler::report_warning(&handler, warning, |_| Err(&failure));
        assert!(std::ptr::eq(result.unwrap_err(), &failure));
        assert_eq!(handler.borrow().loaded_bytes(), payload.len());
        IipHandler::report_warning(&handler, warning, |_| Ok::<_, ()>(())).unwrap();
        assert_eq!(handler.borrow().loaded_bytes(), 0);
    }
}

#[test]
fn successful_warning_releases_nested_live_transfer_but_failure_retains_it() {
    for throws in [false, true] {
        let handler = handler();
        let effects = write(&handler, "File=inline=1:AAAA");
        let [IipEffect::Warning(warning)] = effects.as_slice() else {
            panic!("{effects:?}")
        };
        let result = IipHandler::report_warning(&handler, warning, |_| {
            assert_eq!(handler.borrow().loaded_bytes(), 4);
            assert!(write(&handler, "MultipartFile=inline=1").is_empty());
            assert!(write(&handler, "FilePart=AQIDBAUG").is_empty());
            assert_eq!(handler.borrow().loaded_bytes(), 8);
            if throws {
                Err("warning failed")
            } else {
                Ok(())
            }
        });
        assert_eq!(result.is_err(), throws);
        assert!(handler.borrow().is_multipart());
        assert_eq!(handler.borrow().loaded_bytes(), if throws { 8 } else { 0 });
    }
}

#[test]
fn warning_reset_reentry_does_not_borrow_the_controller_across_callback() {
    let handler = handler();
    let effects = write(&handler, "File=inline=1:AAAA");
    let [IipEffect::Warning(warning)] = effects.as_slice() else {
        panic!("{effects:?}")
    };
    IipHandler::report_warning(&handler, warning, |_| {
        handler.borrow_mut().reset();
        Ok::<_, ()>(())
    })
    .unwrap();
    assert_eq!(handler.borrow().loaded_bytes(), 0);
}
