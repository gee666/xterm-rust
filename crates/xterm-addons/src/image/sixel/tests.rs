// Copyright (c) 2021 Joerg Breitbart; 2020-2023 xterm.js authors. MIT license.
use super::*;
use xterm_core::buffer::{
    constants::{attributes as color, fg_flags},
    AttributeData,
};
use xterm_core::parser::params::Params;

fn decode(data: &str, truncate: bool) -> SixelDecoder {
    let mut decoder = SixelDecoder::new(DecoderOptions::default()).unwrap();
    decoder.init(0, None, 4096.0, truncate);
    decoder
        .decode(&data.encode_utf16().collect::<Vec<_>>())
        .unwrap();
    decoder
}

#[test]
fn native_pixels_and_chunk_boundaries() {
    let input = "#1;2;100;0;0!2~-$#2;1;0;50;100@";
    let mut whole = decode(input, true);
    let expected = whole.image().unwrap();
    assert_eq!((expected.width(), expected.height()), (2, 7));
    assert_eq!(&expected.pixels()[..8], &[255, 0, 0, 255, 255, 0, 0, 255]);
    for step in 1..=input.len() {
        let mut streamed = SixelDecoder::new(DecoderOptions::default()).unwrap();
        streamed.init(0, None, 4096.0, true);
        for chunk in input.as_bytes().chunks(step) {
            streamed.decode(chunk).unwrap();
        }
        assert_eq!(streamed.image().unwrap(), expected, "chunk size {step}");
    }
}

#[test]
fn dynamic_ragged_bands_and_overdraw() {
    let mut decoder = decode("!2~-!1~-", false);
    let image = decoder.image().unwrap();
    assert_eq!((image.width(), image.height()), (2, 12));
    for y in 0..12 {
        for x in 0..2 {
            let offset = (y * 2 + x) * 4;
            let pixel = &image.pixels()[offset..offset + 4];
            assert_eq!(pixel, if y < 6 || x == 0 { &[255; 4] } else { &[0; 4] });
        }
    }
    let mut decoder = decode("~$#1@", false);
    let image = decoder.image().unwrap();
    assert_eq!((image.width(), image.height()), (1, 6));
    assert_eq!(&image.pixels()[..4], &[51, 51, 204, 255]);
    assert!(image.pixels()[4..].iter().all(|&value| value == 255));
}

#[test]
fn raster_modes_and_source_numeric_edges() {
    let mut decoder = decode("\"1;1;3;7#1~-", true);
    let image = decoder.image().unwrap();
    assert_eq!((image.width(), image.height()), (3, 7));
    assert_eq!(decoder.mode(), SixelMode::Truncated);
    assert_eq!(&image.pixels()[..4], &[51, 51, 204, 255]);
    assert_eq!(&image.pixels()[4..12], &[0; 8]);
    assert_eq!(decoder.properties().unwrap().raster.width, 3);
    let decoder = decode("\"1;1;3;7", true);
    assert_eq!(decoder.mode(), SixelMode::Undecided);
    let decoder = decode("\"1;1;0;32#", true);
    assert_eq!((decoder.width(), decoder.height()), (0, 32));
    let decoder = decode("!2147483648~", false);
    assert_eq!((decoder.width(), decoder.height()), (0, 0));
    let mut decoder = decode("", false);
    assert_eq!(
        decoder.decode(&b"!4294967295~"[..]),
        Err(DecodeError::MemoryAccess)
    );
}

#[test]
fn palette_persists_and_hls_preserves_wasm_arithmetic() {
    let mut decoder = decode("#1;1;0;1000;100~", false);
    assert_eq!(decoder.pixels().unwrap()[0], 0xffff_ffed);
    decoder.init(0, None, 4096.0, true);
    decoder.decode(&b"#1~"[..]).unwrap();
    assert_eq!(decoder.pixels().unwrap()[0], 0xffff_ffed);
    decoder.init(0, None, 4096.0, true);
    decoder.decode(&b"#1;1;0;50;1000~"[..]).unwrap();
    assert_eq!(decoder.pixels().unwrap()[0], 0xff7b_0000);
    decoder.clear_palette();
    decoder.init(0, Some(&ADDON_PALETTE), 4096.0, true);
    assert_eq!(decoder.palette()[1], 0xffcc_3333);
    assert!(decoder.palette()[256..].iter().all(|&value| value == 0));
}

#[test]
fn fractional_nan_and_zero_limits_follow_source() {
    for limit in [Some(0.0), Some(f64::NAN), Some(f64::INFINITY), None] {
        let mut decoder = SixelDecoder::new(DecoderOptions {
            memory_limit: limit,
            ..DecoderOptions::default()
        })
        .unwrap();
        decoder.init(0, None, 4096.0, true);
        decoder.decode(&b"\"1;1;2;1~"[..]).unwrap();
        assert_eq!(decoder.image().unwrap().pixels().len(), 8);
    }
    let mut decoder = SixelDecoder::new(DecoderOptions {
        memory_limit: Some(0.5),
        ..DecoderOptions::default()
    })
    .unwrap();
    decoder.init(0, None, 4096.0, true);
    assert_eq!(
        decoder.decode(&b"\"1;1;2;1~"[..]),
        Err(DecodeError::MemoryLimit)
    );
    assert_eq!(decoder.mode(), SixelMode::Undecided);
    let mut decoder = SixelDecoder::new(DecoderOptions {
        palette_limit: f64::NAN,
        ..DecoderOptions::default()
    })
    .unwrap();
    decoder.init(0, None, f64::NAN, true);
    decoder.decode(&b"~"[..]).unwrap();
    assert_eq!(decoder.palette().len(), 0);
    assert_eq!(decoder.decode(&b"#0~"[..]), Err(DecodeError::PaletteZero));
}

#[test]
fn readiness_stale_p2_abort_and_height_only_end() {
    let mut handler = SixelHandler::pending();
    let mut params = Params::default();
    params.add_param(0).unwrap();
    params.add_param(1).unwrap();
    params.reset();
    params.add_param(0).unwrap();
    handler
        .hook(&params, 4096.0, || {
            Err("must not access background while pending")
        })
        .unwrap();
    assert_eq!(handler.put(&[b'~' as u32], 0.0), SixelPut::Continue);
    assert_eq!(handler.size(), 0.0);
    handler.complete_loading(
        SixelDecoder::new(SixelHandler::decoder_options(16_777_216.0, 4096.0)).unwrap(),
    );
    handler
        .hook(&params, 4096.0, || Err("stale P2=1 must skip background"))
        .unwrap();
    assert_eq!(handler.properties().unwrap().unwrap().fill_color, 0);
    assert_eq!(
        handler.put(&[b'~' as u32], 0.0),
        SixelPut::Warning(SixelWarning::TooMuchData)
    );
    assert!(
        !handler.aborted(),
        "source warns before setting abort and releasing"
    );
    handler.finish_abort();
    assert!(handler.aborted());
    assert_eq!(handler.end(true).unwrap(), SixelEnd::Handled);
    handler
        .hook(&params, 4096.0, || Ok::<_, ()>(0xff00_0000))
        .unwrap();
    let data: Vec<u32> = "\"1;1;0;32#".bytes().map(u32::from).collect();
    assert_eq!(handler.put(&data, 1000.0), SixelPut::Continue);
    assert_eq!(
        handler.end(true).unwrap(),
        SixelEnd::AdvanceCursor { height: 32 }
    );
}

#[test]
fn active_inverse_background_and_missing_theme() {
    let ansi = [0x1020_3040, 0x5060_7080];
    let colors = ActiveColors {
        foreground: 0x1122_3344,
        background: 0x5566_7788,
        ansi: &ansi,
    };
    let mut attr = AttributeData::default();
    assert_eq!(active_background(&attr, None).unwrap(), 0);
    assert_eq!(active_background(&attr, Some(colors)).unwrap(), 0x8877_6655);
    attr.fg |= fg_flags::INVERSE;
    assert_eq!(active_background(&attr, Some(colors)).unwrap(), 0x4433_2211);
    attr.fg = fg_flags::INVERSE | color::CM_RGB | 0x123456;
    assert_eq!(active_background(&attr, Some(colors)).unwrap(), 0xff56_3412);
    attr.fg = fg_flags::INVERSE | color::CM_P256 | 1;
    assert_eq!(active_background(&attr, Some(colors)).unwrap(), 0x8070_6050);
    attr.fg = fg_flags::INVERSE | color::CM_P256 | 2;
    assert_eq!(
        active_background(&attr, Some(colors)),
        Err(BackgroundError { palette_index: 2 })
    );
}
