use xterm_addons::image::{
    iip::{
        geometry::IipGeometry,
        handler::IipHandler,
        types::{IipEffect, IipOptions},
    },
    NameDecodePolicy,
};

fn sequence(handler: &mut IipHandler, text: &str) -> Vec<IipEffect> {
    handler.start();
    let data: Vec<_> = text.chars().map(u32::from).collect();
    handler.put(&data, 0, data.len()).unwrap();
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
fn supplied_encoded_cap_controls_payload_instead_of_typed_decoded_size() {
    for (cap, accepted) in [(0.0, false), (3.0, false), (4.0, true), (8.0, true)] {
        let mut handler = IipHandler::new_with_encoded_limit(
            IipOptions {
                size_limit: 0.0,
                ..IipOptions::default()
            },
            NameDecodePolicy::NodeBuffer,
            cap,
        )
        .unwrap();
        sequence(&mut handler, "File=inline=1:AAAA");
        assert_eq!(!handler.is_aborted(), accepted);
        assert_eq!(handler.loaded_bytes(), if accepted { 4 } else { 0 });
        handler.reset();
        assert_eq!(handler.loaded_bytes(), 0);
    }
}

#[test]
fn nonfinite_and_negative_caps_retain_construction_and_reports_but_not_unsafe_decode() {
    for cap in [f64::NAN, f64::NEG_INFINITY, -1.0] {
        let mut handler = IipHandler::new_with_encoded_limit(
            IipOptions::default(),
            NameDecodePolicy::NodeBuffer,
            cap,
        )
        .unwrap();
        assert!(matches!(
            sequence(&mut handler, "ReportCellSize").as_slice(),
            [IipEffect::Report(_)]
        ));
        handler.reset();
        handler.start();
        let data: Vec<_> = "File=inline=1:AAAA".chars().map(u32::from).collect();
        assert!(handler.put(&data, 0, data.len()).is_err());
        assert_eq!(handler.loaded_bytes(), 0);
    }
}

#[test]
fn decoder_constructor_rejects_caps_above_the_reference_address_bound() {
    for cap in [f64::INFINITY, 4_294_901_761.0] {
        assert!(IipHandler::new_with_encoded_limit(
            IipOptions::default(),
            NameDecodePolicy::NodeBuffer,
            cap,
        )
        .is_err());
    }
    IipHandler::new_with_encoded_limit(
        IipOptions::default(),
        NameDecodePolicy::NodeBuffer,
        4_294_901_760.0,
    )
    .unwrap();
}
