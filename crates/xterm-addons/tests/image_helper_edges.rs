use xterm_addons::image::header::{HeaderParser, HeaderState, HeaderValue, ParseOutcome};
use xterm_addons::image::metrics::{image_type, ImageType, UNSUPPORTED_TYPE};
use xterm_addons::image::types::{RasterError, RgbaImage};
use xterm_addons::image::NameDecodePolicy;

fn parse(parser: &mut HeaderParser, text: &str) -> ParseOutcome {
    let input: Vec<_> = text.encode_utf16().map(u32::from).collect();
    parser.parse(&input, 0, input.len()).unwrap()
}
#[test]
fn node_buffer_truncates_ucs2_base64_but_browser_atob_rejects() {
    // Retained seed443424806 counterexample: low bytes form "ar==", or byte106.
    let mut data: Vec<_> = "File=name=".bytes().map(u32::from).collect();
    data.extend([3868278113, 94, 0, 920964722, 61, 61, 104721, 58]);
    let mut node = HeaderParser::default();
    assert_eq!(
        node.parse(&data, 0, data.len()).unwrap(),
        ParseOutcome::Payload(data.len())
    );
    assert_eq!(
        node.fields.get_ascii("name"),
        Some(&HeaderValue::Text(vec![106]))
    );
    let mut browser = HeaderParser::new(NameDecodePolicy::BrowserAtob);
    assert_eq!(
        browser.parse(&data, 0, data.len()).unwrap(),
        ParseOutcome::Rejected
    );
    assert_eq!(browser.state, HeaderState::Abort);
}
#[test]
fn filename_bom_empty_padding_and_policy_are_deliberate() {
    let mut node = HeaderParser::default();
    parse(&mut node, "File=name=77u/YQ==:");
    assert_eq!(
        node.fields.get_ascii("name"),
        Some(&HeaderValue::Text(vec![0xfeff, 97]))
    );
    let mut browser = HeaderParser::new(NameDecodePolicy::BrowserAtob);
    parse(&mut browser, "File=name=77u/YQ==:");
    assert_eq!(
        browser.fields.get_ascii("name"),
        Some(&HeaderValue::Text(vec![97]))
    );
    browser.reset();
    assert_eq!(parse(&mut browser, "File=name=A:"), ParseOutcome::Rejected);
    node.reset();
    assert!(matches!(
        parse(&mut node, "File=name=A:"),
        ParseOutcome::Payload(_)
    ));
    assert_eq!(
        node.fields.get_ascii("name"),
        Some(&HeaderValue::Text(vec![]))
    );
}
#[test]
fn exact_capacity_abort_partial_state_and_stale_start_guard() {
    let mut parser = HeaderParser::default();
    let exact = format!("File=x={}:tail", "1".repeat(1024));
    assert!(matches!(
        parse(&mut parser, &exact),
        ParseOutcome::Payload(_)
    ));
    assert_eq!(
        parser.fields.get_ascii("x"),
        Some(&HeaderValue::Codepoints(vec![49; 1024]))
    );
    parser.reset();
    let overflow = format!("File=x={}:tail", "1".repeat(1025));
    assert_eq!(parse(&mut parser, &overflow), ParseOutcome::Rejected);
    assert_eq!(parser.fields.get_ascii("x"), Some(&HeaderValue::Null));
    assert_eq!(
        parser.fields.get_ascii("type"),
        Some(&HeaderValue::Number(0.0))
    );
    parser.reset();
    assert_eq!(
        parse(&mut parser, "123456789012345"),
        ParseOutcome::NeedMore
    );
    assert_eq!(parse(&mut parser, "="), ParseOutcome::Rejected);
    assert_eq!(parser.state, HeaderState::Start);
    assert!(parser.fields.entries().is_empty());
    assert_eq!(parser.end(), ParseOutcome::Rejected);
    assert_eq!(parser.state, HeaderState::Abort);
}
#[test]
fn source_prototype_keys_and_unknown_value_copy() {
    let mut parser = HeaderParser::default();
    parse(
        &mut parser,
        "File=toString=x;valueOf=x;constructor=12;1=x;0=y;01=z:",
    );
    assert_eq!(
        parser.fields.get_ascii("toString"),
        Some(&HeaderValue::Text(
            "[object Object]".encode_utf16().collect()
        ))
    );
    assert_eq!(
        parser.fields.get_ascii("valueOf"),
        Some(&HeaderValue::DecoderObject)
    );
    assert_eq!(
        parser.fields.get_ascii("constructor"),
        Some(&HeaderValue::Codepoints(vec![49, 50]))
    );
    let entries = parser.fields.entries();
    assert_eq!(entries[0].0, [48]);
    assert_eq!(entries[1].0, [49]);
    let saved = parser.fields.clone();
    parser.reset();
    assert!(parser.buffer().iter().all(|v| *v == 0));
    assert!(parser.current_key().is_empty());
    assert_eq!(
        saved.get_ascii("constructor"),
        Some(&HeaderValue::Codepoints(vec![49, 50]))
    );
    parse(&mut parser, "File=__proto__=x:");
    assert_eq!(parser.state, HeaderState::Abort);
    assert!(parser.fields.has_null_prototype());
    assert!(parser.fields.get_ascii("__proto__").is_none());
}
#[test]
fn checked_range_and_pixel_allocation_boundaries() {
    assert_eq!(ParseOutcome::NeedMore.source_code().unwrap(), -2);
    assert_eq!(ParseOutcome::Rejected.source_code().unwrap(), -1);
    assert_eq!(ParseOutcome::Payload(0).source_code().unwrap(), 0);
    #[cfg(target_pointer_width = "64")]
    assert!(ParseOutcome::Payload(usize::MAX).source_code().is_err());
    #[cfg(target_pointer_width = "32")]
    assert_eq!(
        ParseOutcome::Payload(usize::MAX).source_code().unwrap(),
        4_294_967_295
    );
    let mut parser = HeaderParser::default();
    assert!(parser.parse(&[1], 0, 2).is_err());
    assert!(parser.parse(&[1], 1, 0).is_err());
    assert_eq!(parser.state, HeaderState::Start);
    assert!(parser.fields.entries().is_empty());
    assert_eq!(
        RgbaImage::new(1, 1, vec![0; 3]),
        Err(RasterError::PixelLength {
            expected: 4,
            actual: 3
        })
    );
    assert_eq!(
        RgbaImage::new(u32::MAX, u32::MAX, vec![]),
        Err(RasterError::DimensionsOverflow)
    );
    let empty = RgbaImage::new(0, u32::MAX, vec![]).unwrap();
    assert_eq!(empty.width(), 0);
    assert_eq!(empty.height(), u32::MAX);
    assert!(empty.pixels().is_empty());
    assert_eq!(
        RgbaImage::new(1, 1, vec![1, 2, 3, 4])
            .unwrap()
            .into_pixels(),
        [1, 2, 3, 4]
    );
}
#[test]
fn signed_dimensions_and_avif_scan_limit_are_source_exact() {
    for size in 0..32 {
        assert_eq!(image_type(&vec![0; size]), UNSUPPORTED_TYPE);
    }
    let mut png = vec![0; 32];
    png[..8].copy_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);
    png[12..16].copy_from_slice(b"IHDR");
    png[16..20].copy_from_slice(&u32::MAX.to_be_bytes());
    png[20..24].copy_from_slice(&0x80000000_u32.to_be_bytes());
    let metrics = image_type(&png);
    assert_eq!(metrics.mime, ImageType::Png);
    assert_eq!((metrics.width, metrics.height), (-1, i32::MIN));
    let mut avif = vec![0; 1050];
    avif[4..12].copy_from_slice(b"ftypavif");
    avif[1024..1028].copy_from_slice(b"ispe");
    avif[1032..1036].copy_from_slice(&10_i32.to_be_bytes());
    avif[1036..1040].copy_from_slice(&20_i32.to_be_bytes());
    assert_eq!(image_type(&avif), UNSUPPORTED_TYPE);
    avif[1023..1027].copy_from_slice(b"ispe");
    avif[1031..1035].copy_from_slice(&10_i32.to_be_bytes());
    avif[1035..1039].copy_from_slice(&20_i32.to_be_bytes());
    let metrics = image_type(&avif);
    assert_eq!(metrics.mime, ImageType::Avif);
    assert_eq!((metrics.width, metrics.height), (10, 20));
}
