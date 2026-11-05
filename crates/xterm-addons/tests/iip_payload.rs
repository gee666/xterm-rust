use xterm_addons::image::{
    iip::{
        geometry::IipGeometry,
        handler::IipHandler,
        types::{IipEffect, IipOptions},
    },
    NameDecodePolicy,
};

const PNG: &str = "iVBORw0KGgoAAAAASUhEUgAAAAIAAAABAAAAAAAAAAA=";
fn write(handler: &mut IipHandler, text: &str) -> Vec<IipEffect> {
    handler.start();
    let units: Vec<_> = text.chars().map(u32::from).collect();
    handler.put(&units, 0, units.len()).unwrap();
    handler
        .end(
            true,
            &IipGeometry {
                columns: 80.0,
                rows: 24.0,
                render: None,
                device_pixel_ratio: None,
            },
        )
        .unwrap()
}
#[test]
fn display_request_retains_payload_until_successful_host_preparation() {
    let mut handler = IipHandler::new(IipOptions::default(), NameDecodePolicy::NodeBuffer).unwrap();
    let effects = write(&mut handler, &format!("File=inline=1:{PNG}"));
    let [IipEffect::Display(request)] = effects.as_slice() else {
        panic!("missing image")
    };
    assert_eq!(handler.loaded_bytes(), PNG.len());
    // A host failure drops its owned request without releasing protocol state.
    let bytes = request.bytes.clone();
    drop(effects);
    assert_eq!(handler.loaded_bytes(), PNG.len());
    handler.release_payload();
    assert_eq!(handler.loaded_bytes(), 0);
    assert_eq!(bytes.len(), 32);
}
#[test]
fn outer_host_completion_releases_live_nested_payload_not_an_old_snapshot() {
    let mut handler = IipHandler::new(IipOptions::default(), NameDecodePolicy::NodeBuffer).unwrap();
    let effects = write(&mut handler, &format!("File=inline=1:{PNG}"));
    assert_eq!(handler.loaded_bytes(), PNG.len());
    for text in ["MultipartFile=inline=1", "FilePart=AAAA"] {
        assert!(write(&mut handler, text).is_empty());
    }
    assert_eq!(handler.loaded_bytes(), 4);
    handler.release_payload();
    assert_eq!(handler.loaded_bytes(), 0);
    assert!(handler.is_multipart());
    let [IipEffect::Display(request)] = effects.as_slice() else {
        panic!("missing image")
    };
    assert_eq!(request.bytes.len(), 32);
}
