use xterm_addons::image::header::HeaderState;
use xterm_addons::image::iip::geometry::{CellSize, IipGeometry, RenderMetrics, SizeRequest};
use xterm_addons::image::iip::handler::IipHandler;
use xterm_addons::image::iip::reports::{cell_size_report, fixed_three};
use xterm_addons::image::iip::types::{
    EndPreparation, GeometryPhase, IipEffect, IipError, IipOptions,
};
use xterm_addons::image::NameDecodePolicy;

// Metadata-only PNG header. Tests verify an explicit decode request, not pixels
// or successful image rendering. Real fixtures and hosts belong to integration.
const PNG: &str = "iVBORw0KGgoAAAAASUhEUgAAAAIAAAABAAAAAAAAAAA=";
fn geometry() -> IipGeometry {
    IipGeometry {
        columns: 80.0,
        rows: 24.0,
        render: None,
        device_pixel_ratio: None,
    }
}
fn handler() -> IipHandler {
    IipHandler::new(IipOptions::default(), NameDecodePolicy::NodeBuffer).unwrap()
}
fn put(handler: &mut IipHandler, text: &str) {
    let units: Vec<_> = text.encode_utf16().map(u32::from).collect();
    handler.put(&units, 0, units.len()).unwrap();
}
fn sequence(handler: &mut IipHandler, text: &str, success: bool) -> Vec<IipEffect> {
    handler.start();
    put(handler, text);
    handler.end(success, &geometry()).unwrap()
}

#[test]
fn file_metadata_request_defers_real_decode_and_size_metadata_is_informative() {
    let mut handler = handler();
    for size in ["", "size=1;", "size=999999;"] {
        let effects = sequence(
            &mut handler,
            &format!("File=inline=1;{size}width=1:{PNG}"),
            true,
        );
        let [IipEffect::Display(request)] = effects.as_slice() else {
            panic!("missing explicit display request: {effects:?}")
        };
        assert_eq!((request.source.width, request.source.height), (2, 1));
        assert_eq!((request.target.width, request.target.height), (7.0, 3.0));
        assert_eq!(request.bytes.len(), 32);
        assert_eq!(handler.loaded_bytes(), 0);
    }
}
#[test]
fn inline_disabled_failure_and_overflow_recover_at_next_start() {
    let mut handler = handler();
    assert!(sequence(&mut handler, &format!("File=inline=0:{PNG}"), true).is_empty());
    assert!(handler.is_aborted());
    assert!(sequence(&mut handler, &format!("File=inline=1:{PNG}"), false).is_empty());
    let effects = sequence(&mut handler, "File=inline=1:%%%", true);
    assert!(
        matches!(effects.as_slice(),[IipEffect::Warning(text)] if text=="IIP: error during BASE64 decoding")
    );
    assert!(matches!(
        sequence(&mut handler, &format!("File=inline=1:{PNG}"), true).as_slice(),
        [IipEffect::Display(_)]
    ));
    let mut limited = IipHandler::new(
        IipOptions {
            size_limit: 1,
            ..IipOptions::default()
        },
        NameDecodePolicy::NodeBuffer,
    )
    .unwrap();
    assert!(sequence(&mut limited, "File=inline=1:AAAA", true).is_empty());
    assert!(limited.is_aborted());
}
#[test]
fn multipart_stream_assembles_only_on_file_end_and_file_replaces_pending() {
    let mut handler = handler();
    assert!(sequence(&mut handler, "MultipartFile=inline=1", true).is_empty());
    assert!(handler.is_multipart());
    assert!(sequence(&mut handler, &format!("FilePart={}", &PNG[..20]), false).is_empty());
    assert!(sequence(&mut handler, &format!("FilePart={}", &PNG[20..]), true).is_empty());
    let effects = sequence(&mut handler, "FileEnd", true);
    assert!(matches!(effects.as_slice(),[IipEffect::Display(request)] if request.bytes.len()==32));
    assert!(!handler.is_multipart());
    assert!(sequence(&mut handler, "FileEnd", true).is_empty());
    sequence(&mut handler, "MultipartFile=inline=1", true);
    sequence(&mut handler, "FilePart=AAAA", true);
    assert!(matches!(
        sequence(&mut handler, &format!("File=inline=1:{PNG}"), true).as_slice(),
        [IipEffect::Display(_)]
    ));
    assert!(!handler.is_multipart());
}
#[test]
fn reset_retains_source_multipart_flags_but_releases_decoder_bytes() {
    let mut handler = handler();
    sequence(&mut handler, "MultipartFile=inline=1", true);
    sequence(&mut handler, "FilePart=AAAA", true);
    assert!(handler.loaded_bytes() > 0);
    handler.reset();
    assert!(handler.is_multipart());
    assert_eq!(handler.loaded_bytes(), 0);
    assert_eq!(handler.header_parser().state, HeaderState::Start);
    let effects = sequence(&mut handler, "FileEnd", true);
    assert!(matches!(effects.as_slice(), [IipEffect::Warning(_)]));
}
#[test]
fn report_cell_size_uses_css_canvas_not_the_cell_field_or_success_flag() {
    let mut handler = handler();
    let effects = sequence(&mut handler, "ReportCellSize", false);
    let [IipEffect::Report(report)] = effects.as_slice() else {
        panic!("missing size report")
    };
    assert_eq!(
        String::from_utf16(report).unwrap(),
        "\x1b]1337;ReportCellSize=14.000;7.000;1.000\x1b\\"
    );
    let geometry = IipGeometry {
        columns: 1.0,
        rows: 1.0,
        render: Some(RenderMetrics {
            cell: CellSize {
                width: 999.0,
                height: 999.0,
            },
            canvas: CellSize {
                width: 2.0625,
                height: 1.0625,
            },
        }),
        device_pixel_ratio: Some(0.0625),
    };
    assert_eq!(
        String::from_utf16(&cell_size_report(&geometry)).unwrap(),
        "\x1b]1337;ReportCellSize=1.063;2.063;0.063\x1b\\"
    );
}
#[test]
fn original_and_resized_pixel_limit_equalities_reject_display_requests() {
    let mut original = IipHandler::new(
        IipOptions {
            pixel_limit: 2.0,
            ..IipOptions::default()
        },
        NameDecodePolicy::NodeBuffer,
    )
    .unwrap();
    let effects = sequence(&mut original, &format!("File=inline=1:{PNG}"), true);
    assert!(
        matches!(effects.as_slice(),[IipEffect::Warning(text)] if text=="IIP: image dimension issue 2x1")
    );
    let mut resized = IipHandler::new(
        IipOptions {
            pixel_limit: 21.0,
            ..IipOptions::default()
        },
        NameDecodePolicy::NodeBuffer,
    )
    .unwrap();
    assert!(sequence(&mut resized, &format!("File=inline=1;width=1:{PNG}"), true).is_empty());
}
#[test]
fn dimensions_use_source_width_precedence_and_no_auto_upscale() {
    let geometry = geometry();
    let mut request = SizeRequest::default();
    assert_eq!(
        geometry.resize(2.0, 1.0, &request),
        CellSize {
            width: 2.0,
            height: 1.0
        }
    );
    request.width = "20".encode_utf16().collect();
    request.height = "5".encode_utf16().collect();
    assert_eq!(
        geometry.resize(200.0, 100.0, &request),
        CellSize {
            width: 140.0,
            height: 70.0
        }
    );
    request.preserve_aspect_ratio = 0.0;
    assert_eq!(
        geometry.resize(200.0, 100.0, &request),
        CellSize {
            width: 140.0,
            height: 70.0
        }
    );
    request.width = "320px".encode_utf16().collect();
    request.height = "160px".encode_utf16().collect();
    assert_eq!(
        geometry.resize(200.0, 100.0, &request),
        CellSize {
            width: 320.0,
            height: 160.0
        }
    );
    request.width = "auto".encode_utf16().collect();
    request.height = "auto".encode_utf16().collect();
    assert_eq!(
        geometry.resize(1120.0, 644.0, &request),
        CellSize {
            width: 560.0,
            height: 322.0
        }
    );
}
#[test]
fn prepare_end_skips_foreign_geometry_for_early_consumed_paths() {
    for text in [
        "File=inline=0:AAAA",
        "FilePart=AAAA",
        "MultipartFile=inline=1",
        "FileEnd",
        "File=inline=1:%%%",
    ] {
        let mut handler = handler();
        handler.start();
        put(&mut handler, text);
        assert!(
            matches!(
                handler.prepare_end(true).unwrap(),
                EndPreparation::Complete(_)
            ),
            "{text}"
        );
    }
    let mut handler = handler();
    handler.start();
    put(&mut handler, "ReportCellSize");
    let EndPreparation::Geometry(token) = handler.prepare_end(false).unwrap() else {
        panic!("report requires host geometry")
    };
    assert_eq!(token.phase(), GeometryPhase::CellReport);
}

#[test]
fn geometry_completion_releases_controller_borrow_and_rejects_foreign_tokens() {
    let mut first = handler();
    first.start();
    put(&mut first, &format!("File=inline=1:{PNG}"));
    let EndPreparation::Geometry(token) = first.prepare_end(true).unwrap() else {
        panic!("image requires geometry")
    };
    assert_eq!(token.phase(), GeometryPhase::Resize);
    assert_eq!(first.loaded_bytes(), PNG.len());
    // A foreign getter can reset the controller before completion. Source then
    // reads current decoder bytes, not a stale pre-getter copy.
    first.reset();
    let effects = first.finish_geometry(token, &geometry()).unwrap();
    assert!(
        matches!(effects.as_slice(), [IipEffect::Display(request)] if request.bytes.is_empty())
    );
    first.start();
    put(&mut first, "ReportCellSize");
    let EndPreparation::Geometry(token) = first.prepare_end(true).unwrap() else {
        panic!("missing geometry phase")
    };
    let mut second = handler();
    assert!(matches!(
        second.finish_geometry(token, &geometry()),
        Err(IipError::ForeignEndToken)
    ));
}

#[test]
fn fixed_three_preserves_source_ties_subnormals_and_special_numbers() {
    for (number, expected) in [
        (0.0, "0.000"),
        (-0.0, "0.000"),
        (1.0625, "1.063"),
        (-1.0625, "-1.063"),
        (1.0005, "1.000"),
        (f64::from_bits(1), "0.000"),
        (-f64::from_bits(1), "-0.000"),
        (1e21, "1e+21"),
        (f64::NAN, "NaN"),
        (f64::INFINITY, "Infinity"),
        (f64::NEG_INFINITY, "-Infinity"),
    ] {
        assert_eq!(fixed_three(number), expected, "{number:?}");
    }
}
