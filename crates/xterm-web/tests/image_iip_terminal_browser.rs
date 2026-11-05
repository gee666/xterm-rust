#![cfg(target_arch = "wasm32")]
#[path = "image_iip_terminal_support/failures.rs"]
mod failures;
#[path = "image_iip_terminal_support/mod.rs"]
mod support;
use support::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

fn tiny_qoi() -> Vec<u8> {
    let mut bytes = b"qoif\0\0\0\x02\0\0\0\x02\x04\0".to_vec();
    for color in [
        [255, 0, 0, 255],
        [0, 255, 0, 255],
        [0, 0, 255, 255],
        [255, 255, 255, 255],
    ] {
        bytes.push(255);
        bytes.extend(color);
    }
    bytes.extend([0, 0, 0, 0, 0, 0, 0, 1]);
    bytes
}

#[wasm_bindgen_test(async)]
async fn streaming_qoi_real_parser_tiles_cells_and_pixels_match_source() {
    let live = Live::new(true);
    let source = Source::new(true);
    let text = format!(
        "\x1b[31;4mA{}Z",
        sequence(&tiny_qoi(), "width=2px;height=2px;preserveAspectRatio=0")
    );
    let mut seed = 0x1337365u32;
    let mut position = 0;
    while position < text.len() {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let end = (position + 1 + (seed as usize % 97)).min(text.len());
        assert!(!live.begin(&text[position..end]), "QOI must not pause");
        assert!(!source.begin(&text[position..end]));
        position = end;
    }
    same_state(&live, &source);
    assert_eq!(
        pixels(&live.image(1, 0)),
        [255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255]
    );
    assert_eq!(pixels(&live.image(1, 0)), pixels(&source.image(1, 0)));
    live.storage.render(0, 9).unwrap();
    call(
        &get(&source.0, "store"),
        "render",
        &[eval("({start:0,end:9})")],
    );
    let canvas = live
        .renderer
        .canvas(xterm_addons::image::storage::ImageLayer::Top)
        .unwrap()
        .unwrap();
    let source_context = call(
        &get(&get(&source.0, "renderer"), "_layers"),
        "get",
        &["top".into()],
    );
    assert_eq!(pixels(&canvas), pixels(&get(&source_context, "canvas")));
}

#[wasm_bindgen_test(async)]
async fn actual_png_and_resized_qoi_pause_before_trailing_print_and_match_pixels() {
    for (bytes, dimensions) in [
        (PNG.to_vec(), "width=14px;height=10px;preserveAspectRatio=0"),
        (tiny_qoi(), "width=14px;height=10px;preserveAspectRatio=0"),
    ] {
        let live = Live::new(true);
        let source = Source::new(true);
        let text = format!("{}Z", sequence(&bytes, dimensions));
        assert!(live.begin(&text));
        assert!(source.begin(&text));
        assert_eq!(live.added.get(), 0);
        assert_eq!(get(&source.0, "added").as_f64(), Some(0.0));
        same_state(&live, &source);
        live.finish(true).await;
        source.finish().await;
        same_state(&live, &source);
        assert_eq!(live.added.get(), 1);
        assert_eq!(pixels(&live.image(0, 0)), pixels(&source.image(0, 0)));
    }
}

#[wasm_bindgen_test(async)]
async fn multipart_bytewise_st_cancellation_and_disabled_inline_match_source() {
    let live = Live::new(true);
    let source = Source::new(true);
    let encoded = sequence(&tiny_qoi(), "width=2px;height=2px;preserveAspectRatio=0");
    let data = encoded
        .split(':')
        .nth(1)
        .unwrap()
        .trim_end_matches("\x1b\\");
    let split = data.len() / 2;
    let text=format!("\x1b]1337;MultipartFile=inline=1;width=2px;height=2px;preserveAspectRatio=0\x07\x1b]1337;FilePart={}\x07", &data[..split]);
    let text = format!(
        "{text}\x1b]1337;FilePart={}\x1b\\\x1b]1337;FileEnd\x1b\\X",
        &data[split..]
    );
    for byte in text.as_bytes() {
        let part = char::from(*byte).to_string();
        live.write(&part).await;
        source.write(&part).await;
    }
    same_state(&live, &source);
    assert_eq!(live.added.get(), 1);
    assert_eq!(pixels(&live.image(0, 0)), pixels(&source.image(0, 0)));
    for text in [
        "\x1b]1337;File=inline=0:AAAA\x07Y",
        "\x1b]1337;File=inline=1:AAAA\x18Z",
    ] {
        live.write(text).await;
        source.write(text).await;
        same_state(&live, &source);
    }
    assert_eq!(live.added.get(), 1);
}

#[wasm_bindgen_test(async)]
async fn cell_report_uses_live_open_geometry_dpr_and_native_input_route() {
    for open in [false, true] {
        let live = Live::new(open);
        let source = Source::new(open);
        for resize in [false, true] {
            if resize {
                live.terminal.resize(24.0, 8.0).unwrap();
                call(&get(&source.0, "t"), "resize", &[24.into(), 8.into()]);
            }
            live.write("\x1b]1337;ReportCellSize\x07").await;
            source.write("\x1b]1337;ReportCellSize\x07").await;
            let reports = get(&source.0, "replies");
            let reports = js_sys::Array::from(&reports);
            assert_eq!(live.replies.borrow().len(), reports.length() as usize);
            for (index, text) in live.replies.borrow().iter().enumerate() {
                assert_eq!(
                    String::from_utf16(text).unwrap(),
                    reports.get(index as u32).as_string().unwrap()
                );
            }
        }
    }
}

#[wasm_bindgen_test(async)]
async fn unregister_removes_parser_hook_and_reregister_restores_execution() {
    let live = Live::new(true);
    let source = Source::new(true);
    live.registration.dispose();
    call(&get(&source.0, "registration"), "dispose", &[]);
    let text = sequence(&tiny_qoi(), "width=2px;height=2px;preserveAspectRatio=0");
    live.write(&text).await;
    source.write(&text).await;
    assert_eq!(live.added.get(), 0);
    same_state(&live, &source);
    let registration = live.protocol.register().unwrap();
    let register = js_sys::Function::new_with_args(
        "s",
        "return s.t._core._inputHandler._parser.registerOscHandler(1337,s.h)",
    );
    let source_registration = register.call1(&JsValue::UNDEFINED, &source.0).unwrap();
    live.write(&text).await;
    source.write(&text).await;
    same_state(&live, &source);
    registration.dispose();
    call(&source_registration, "dispose", &[]);
}

#[wasm_bindgen_test(async)]
async fn real_browser_write_queue_waits_for_bitmap_and_delivers_completion() {
    let live = Live::new(true);
    let mut callback = None;
    let completion = js_sys::Promise::new(&mut |resolve, _| {
        callback = Some(resolve);
    });
    live.terminal
        .write(
            format!(
                "{}Z",
                sequence(PNG, "width=14px;height=10px;preserveAspectRatio=0")
            )
            .into(),
            callback,
        )
        .unwrap();
    assert_eq!(live.added.get(), 0);
    promise(completion.into()).await;
    assert_eq!(live.added.get(), 1);
    assert_eq!(get(&live.image(0, 0), "width").as_f64(), Some(14.0));
}
