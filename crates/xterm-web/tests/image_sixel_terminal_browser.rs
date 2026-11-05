#![cfg(target_arch = "wasm32")]
#[path = "image_sixel_terminal_support/failures.rs"]
mod failures;
#[path = "image_sixel_terminal_support/mod.rs"]
mod support;
use std::{cell::Cell, rc::Rc};
use support::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_addons::image::{options::ImageOptions, storage::ImageLayer};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
async fn real_parser_raster_cells_palette_and_rendered_pixels_match_source() {
    let (live, source, _) = ready(true).await;
    let text = format!(
        "\x1b[31;4mA{}\x1b[5CZ",
        sequence("\"1;1;14;12#1;2;100;0;0!14~-#2;2;0;100;0!14~", true)
    );
    live.write(&text).await;
    source.write(&text).await;
    same_state(&live, &source);
    assert_eq!(live.added.get(), 1);
    assert_eq!(pixels(&live.image(1, 0)), pixels(&source.image(1, 0)));
    assert_eq!(&pixels(&live.image(1, 0))[..4], &[255, 0, 0, 255]);
    live.storage.render(0, 9).unwrap();
    call(
        &get(&source.0, "store"),
        "render",
        &[eval("({start:0,end:9})")],
    );
    let canvas = live.renderer.canvas(ImageLayer::Top).unwrap().unwrap();
    let context = call(
        &get(&get(&source.0, "renderer"), "_layers"),
        "get",
        &["top".into()],
    );
    assert_eq!(pixels(&canvas), pixels(&get(&context, "canvas")));
}

#[wasm_bindgen_test(async)]
async fn seeded_chunks_palette_persistence_and_cancelled_sequences_match_source() {
    let (live, source, _) = ready(true).await;
    let mut text = String::new();
    for i in 1..=12 {
        text.push_str(&sequence(
            &format!(
                "#{};2;{};{};{}!{}~-#{}!{}~",
                i,
                i * 7,
                i * 3,
                i * 5,
                i + 1,
                i,
                i + 2
            ),
            true,
        ));
        text.push_str(&format!("{}\r\n", i));
    }
    text.push_str("\x1bPq#1!3~\x18Z");
    let mut seed = 0x51370370u32;
    let mut pos = 0;
    while pos < text.len() {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let end = (pos + 1 + seed as usize % 31).min(text.len());
        live.write(&text[pos..end]).await;
        source.write(&text[pos..end]).await;
        pos = end;
    }
    same_state(&live, &source);
    assert_eq!(live.added.get(), 12);
}

#[wasm_bindgen_test(async)]
async fn pending_calls_are_discarded_and_mid_sequence_loading_is_not_replayed() {
    let opts = Rc::new(Cell::new(ImageOptions::default()));
    let live = Live::new(true, opts.clone());
    let source = Source::new(true);
    // Neither decoder is ready in this constructor stack.
    assert!(!live.begin(&sequence("#1!4~", false)));
    assert!(!source.begin(&sequence("#1!4~", false)));
    same_state(&live, &source);
    assert_eq!(live.added.get(), 0);
    assert!(!live.begin("\x1bPq#2"));
    assert!(!source.begin("\x1bPq#2"));
    source.ready().await;
    live.load(opts.get());
    live.write("!3~\x1b\\").await;
    source.write("!3~\x1b\\").await;
    same_state(&live, &source);
    assert_eq!(pixels(&live.image(0, 0)), pixels(&source.image(0, 0)));
}

#[wasm_bindgen_test(async)]
async fn active_background_default_rgb_palette_inverse_and_before_open_match_source() {
    for open in [false, true] {
        for sgr in ["0", "44", "48;2;12;34;56", "31;7", "38;2;3;5;7;7", "7"] {
            let (live, source, _) = ready(open).await;
            let text = format!("\x1b[{sgr}m{}", sequence("\"1;1;2;2?", false));
            live.write(&text).await;
            source.write(&text).await;
            same_state(&live, &source);
            assert_eq!(
                pixels(&live.image(0, 0)),
                pixels(&source.image(0, 0)),
                "sgr={sgr} open={open}"
            );
        }
    }
}

#[wasm_bindgen_test(async)]
async fn empty_width_raster_scroll_policy_and_fallback_metrics_match_source() {
    for open in [false, true] {
        for scrolling in [false, true] {
            let (live, source, opts) = ready(open).await;
            let mut value = opts.get();
            value.sixel_scrolling = scrolling;
            opts.set(value);
            source_option(&source, "sixelScrolling", scrolling.into());
            live.write("\x1b[4;5H").await;
            source.write("\x1b[4;5H").await;
            let text = sequence("---", true);
            live.write(&text).await;
            source.write(&text).await;
            same_state(&live, &source);
            assert_eq!(live.added.get(), 0);
        }
    }
}

#[wasm_bindgen_test(async)]
async fn null_canvas_context_still_inserts_blank_image() {
    let (live, source, _) = ready(true).await;
    let restore=eval("(()=>{const old=HTMLCanvasElement.prototype.getContext;HTMLCanvasElement.prototype.getContext=function(type){if(this.width===2&&this.height===3)return null;return old.apply(this,arguments)};return ()=>HTMLCanvasElement.prototype.getContext=old})()");
    let text = sequence("\"1;1;2;3#1!2~", true);
    live.write(&text).await;
    source.write(&text).await;
    restore
        .unchecked_into::<js_sys::Function>()
        .call0(&JsValue::UNDEFINED)
        .unwrap();
    same_state(&live, &source);
    assert_eq!(live.added.get(), 1);
    assert_eq!(pixels(&live.image(0, 0)), vec![0; 24]);
}

#[wasm_bindgen_test(async)]
async fn warnings_limits_palette_reset_and_bytewise_st_match_source() {
    let (live, source, opts) = ready(true).await;
    let warnings=eval("(()=>{const old=console.warn,log=[];console.warn=(...args)=>log.push(args.map(String));return {log,restore(){console.warn=old}}})()");
    let mut value = opts.get();
    value.sixel_size_limit = 3.0;
    opts.set(value);
    source_option(&source, "sixelSizeLimit", 3.into());
    live.write(&sequence("!20~", true)).await;
    let rust_log = js_sys::JSON::stringify(&get(&warnings, "log")).unwrap();
    call(&get(&warnings, "log"), "splice", &[0.into()]);
    source.write(&sequence("!20~", true)).await;
    let source_log = js_sys::JSON::stringify(&get(&warnings, "log")).unwrap();
    call(&warnings, "restore", &[]);
    assert_eq!(rust_log, source_log);
    same_state(&live, &source);
    assert_eq!(live.added.get(), 0);
    value.sixel_size_limit = 33554432.0;
    opts.set(value);
    source_option(&source, "sixelSizeLimit", 33554432.into());
    // A zero palette limit traps in the source WASM decoder. Both adapters
    // warn and abort instead of inserting partial pixels.
    value.sixel_palette_limit = 0.0;
    opts.set(value);
    source_option(&source, "sixelPaletteLimit", 0.into());
    let warnings = eval("(()=>{const old=console.warn,log=[];console.warn=(...args)=>log.push(args.map(String));return {log,restore(){console.warn=old}}})()");
    live.write(&sequence("#1~", true)).await;
    let rust_log = js_sys::JSON::stringify(&get(&warnings, "log")).unwrap();
    call(&get(&warnings, "log"), "splice", &[0.into()]);
    source.write(&sequence("#1~", true)).await;
    let source_log = js_sys::JSON::stringify(&get(&warnings, "log")).unwrap();
    call(&warnings, "restore", &[]);
    assert_eq!(rust_log, source_log);
    same_state(&live, &source);
    assert_eq!(live.added.get(), 0);
    value.sixel_palette_limit = 4096.0;
    opts.set(value);
    source_option(&source, "sixelPaletteLimit", 4096.into());
    let color = sequence("#1;2;100;0;0!2~", true);
    live.write(&color).await;
    source.write(&color).await;
    live.protocol.reset();
    call(&source.0, "reset", &[]);
    let text = sequence("#1!2~", true);
    for byte in text.bytes() {
        let s = char::from(byte).to_string();
        live.write(&s).await;
        source.write(&s).await;
    }
    same_state(&live, &source);
    assert_eq!(pixels(&live.image(0, 0)), pixels(&source.image(0, 0)));
}

#[wasm_bindgen_test(async)]
async fn large_image_then_palette_reuse_matches_source() {
    let (live, source, opts) = ready(true).await;
    let mut options = opts.get();
    options.sixel_scrolling = false;
    opts.set(options);
    source_option(&source, "sixelScrolling", false.into());
    // This 4MiB image plus fixed decoder pages exceeds MEM_PERMA_LIMIT.
    // The following image must still use the retained custom palette.
    let text = sequence("\"1;1;1024;1024#1;2;100;0;0!1024~", true);
    live.write(&text).await;
    source.write(&text).await;
    same_state(&live, &source);
    assert_eq!(pixels(&live.image(0, 0)), pixels(&source.image(0, 0)));
    let text = sequence("#1!2~", true);
    live.write(&text).await;
    source.write(&text).await;
    same_state(&live, &source);
    let bytes = pixels(&live.image(0, 0));
    assert_eq!(bytes, pixels(&source.image(0, 0)));
    assert_eq!(&bytes[..4], &[255, 0, 0, 255]);
    assert_eq!(live.added.get(), 2);
}

#[wasm_bindgen_test(async)]
async fn unregister_reregister_and_real_browser_queued_write_execute_sixel() {
    let (live, source, _) = ready(true).await;
    assert!(!live.begin("\x1bPq#1"));
    assert!(!source.begin("\x1bPq#1"));
    live.registration.dispose();
    call(&get(&source.0, "registration"), "dispose", &[]);
    live.write("!3~\x1b\\").await;
    source.write("!3~\x1b\\").await;
    same_state(&live, &source);
    assert_eq!(live.added.get(), 0);
    let text = sequence("#1!3~", true);
    live.write(&text).await;
    source.write(&text).await;
    same_state(&live, &source);
    assert_eq!(live.added.get(), 0);
    let token = live.protocol.register().unwrap();
    let registration = js_sys::Function::new_with_args(
        "s",
        "return s.t._core._inputHandler._parser.registerDcsHandler({final:'q'},s.h)",
    )
    .call1(&JsValue::UNDEFINED, &source.0)
    .unwrap();
    let mut complete = None;
    let pending = js_sys::Promise::new(&mut |resolve, _| complete = Some(resolve));
    live.terminal
        .write(format!("{text}Z").into(), complete)
        .unwrap();
    promise(pending.into()).await;
    source.write(&format!("{text}Z")).await;
    same_state(&live, &source);
    assert_eq!(live.added.get(), 1);
    token.dispose();
    call(&registration, "dispose", &[]);
}
