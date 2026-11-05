#![cfg(target_arch = "wasm32")]
#[path = "image_kitty_terminal_support/async_cases.rs"]
mod async_cases;
#[path = "image_kitty_terminal_support/failures.rs"]
mod failures;
#[path = "image_kitty_terminal_support/lifetime.rs"]
mod lifetime;
#[path = "image_kitty_terminal_support/mod.rs"]
mod support;
use support::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use xterm_addons::image::{kitty::handler::KittyOptions, storage::ImageLayer};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
async fn rgba_rgb_unknown_formats_png_and_real_rendered_pixels_match_source() {
    for (format, bytes) in [
        (32, raw()),
        (
            24,
            raw()
                .chunks_exact(4)
                .flat_map(|p| p[..3].to_vec())
                .collect(),
        ),
        (
            17,
            raw()
                .chunks_exact(4)
                .flat_map(|p| p[..3].to_vec())
                .collect(),
        ),
        (100, PNG.to_vec()),
    ] {
        let live = Live::new(true);
        let source = Source::new(true);
        let text = format!(
            "\x1b[31;4mA{}\x1b[5CZ",
            sequence(&format!("a=T,i=7,s=6,v=9,f={format}"), &bytes)
        );
        assert!(live.begin(&text));
        assert!(source.begin(&text));
        live.finish(true).await;
        source.finish().await;
        same_state(&live, &source);
        same_maps(&live, &source).await;
        assert_eq!(live.added.get(), 1);
        assert_eq!(pixels(&live.image(1, 0)), pixels(&source.image(1, 0)));
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
}
#[wasm_bindgen_test(async)]
async fn crop_scale_offsets_bottom_layer_and_cursor_restore_match_source() {
    for open in [false, true] {
        for params in [
            "x=1,y=2,w=4,h=6",
            "c=2",
            "r=2",
            "c=2,r=2,X=2,Y=3",
            "X=3,Y=5",
            "c=2,r=2,C=1,z=-3",
            "x=1,y=2,c=2,r=3,X=2,Y=1,z=2",
        ] {
            let live = Live::new(open);
            let source = Source::new(open);
            let text = format!(
                "\x1b[10;4H{}",
                sequence(&format!("a=T,i=7,s=6,v=9,{params}"), &raw())
            );
            live.write(&text).await;
            source.write(&text).await;
            same_state(&live, &source);
            same_maps(&live, &source).await;
            assert_eq!(live.added.get(), 1, "open={open} params={params}");
            // Placement can scroll, so find the image through its actual tiles.
            let source_image = js_sys::Function::new_with_args("s", "const b=s.t._core.buffer;for(let y=0;y<b.lines.length;y++){for(let x=0;x<20;x++){const image=s.store.getImageAtBufferCell(x,y);if(image)return [x,y,image]}}").call1(&JsValue::UNDEFINED, &source.0).unwrap();
            let x = get(&source_image, "0").as_f64().unwrap() as usize;
            let y = get(&source_image, "1").as_f64().unwrap() as usize;
            assert_eq!(pixels(&live.image(x, y)), pixels(&get(&source_image, "2")));
        }
    }
}
#[wasm_bindgen_test(async)]
async fn zlib_and_constructor_fallback_to_raw_deflate_match_source() {
    for format in ["deflate", "deflate-raw"] {
        // Native invalid-zlib streams reject ignored writer promises in both
        // source and Rust. Exercise the supported synchronous fallback boundary
        // here; retain that separate diagnostic rather than filtering pageerrors.
        let _fallback = (format == "deflate-raw").then(|| {
            let wrapper = js_sys::Function::new_with_args("original", "return function(format){if(format==='deflate')throw new Error('unsupported format');return new original(format)}")
                .call1(&JsValue::UNDEFINED, &get(&js_sys::global(), "DecompressionStream")).unwrap();
            Patch::new(js_sys::global().into(), "DecompressionStream", wrapper)
        });
        let live = Live::new(true);
        let source = Source::new(true);
        let bytes = compressed(&raw(), format).await;
        let text = sequence("a=T,i=2,s=6,v=9,o=z", &bytes);
        live.write(&text).await;
        source.write(&text).await;
        same_state(&live, &source);
        assert_eq!(pixels(&live.image(0, 0)), pixels(&source.image(0, 0)));
    }
}
#[wasm_bindgen_test(async)]
async fn multipart_upload_seeded_chunks_transmit_place_replace_and_delete_match_source() {
    let live = Live::new(true);
    let source = Source::new(true);
    let encoded = payload(&raw());
    let text = format!(
        "\x1b_Ga=T,i=33,s=6,v=9,m=1;{}\x1b\\\x1b_Gm=0;{}\x1b\\Z",
        &encoded[..31],
        &encoded[31..]
    );
    let mut seed = 0x4b373u32;
    let mut pos = 0;
    while pos < text.len() {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let end = (pos + 1 + seed as usize % 13).min(text.len());
        live.write(&text[pos..end]).await;
        source.write(&text[pos..end]).await;
        pos = end;
    }
    same_state(&live, &source);
    same_maps(&live, &source).await;
    for text in [
        sequence("a=t,i=5,s=6,v=9", &raw()),
        "\x1b_Ga=p,i=5,p=9,C=1\x1b\\".into(),
        sequence("a=T,i=5,s=6,v=9", &raw()),
        "\x1b_Ga=d,d=i,i=5\x1b\\".into(),
        "\x1b_Ga=d,d=A\x1b\\".into(),
    ] {
        live.write(&text).await;
        source.write(&text).await;
        same_state(&live, &source);
        same_maps(&live, &source).await;
    }
}
#[wasm_bindgen_test(async)]
async fn protocol_queries_quiet_errors_invalid_images_and_limits_match_source() {
    let live = Live::options(
        true,
        KittyOptions {
            size_limit: 256.0,
            pixel_limit: 128.0,
        },
    );
    let source = Source::options(true, "({kittySizeLimit:256,pixelLimit:128})");
    for text in [
        "\x1b_Ga=q,i=4\x1b\\".into(),
        "\x1b_Ga=q,i=4,q=1\x1b\\".into(),
        "\x1b_Ga=p,i=9,p=3\x1b\\".into(),
        "\x1b_Ga=T,i=7,I=3;AAAA\x1b\\".into(),
        "\x1b_Ga=T,i=7;%%%\x1b\\".into(),
        sequence("a=T,i=7,s=10,v=10", &[0; 4]),
        sequence("a=T,i=7,s=6,v=9,c=4,r=5", &raw()),
        sequence("a=T,i=7,s=6,v=9,x=99", &raw()),
        sequence("a=T,i=7,s=6,v=9", &[0; 300]),
        sequence("a=T,i=7,f=100", &[0; 10]),
        sequence("a=q,i=3,s=6,v=9", &raw()),
    ] {
        live.write(&text).await;
        source.write(&text).await;
        same_state(&live, &source);
        same_maps(&live, &source).await;
    }
}
#[wasm_bindgen_test(async)]
async fn pending_blob_read_is_eager_and_late_image_survives_reset_and_disposal() {
    for dispose in [false, true] {
        let live = Live::new(true);
        let source = Source::new(true);
        let gate = eval("(()=>{const old=Blob.prototype.arrayBuffer, gates=[];Blob.prototype.arrayBuffer=function(){const p=old.call(this);return new Promise(resolve=>gates.push(()=>p.then(resolve)))};return {gates,restore(){Blob.prototype.arrayBuffer=old}}})()");
        let text = sequence("a=T,i=3,s=6,v=9", &raw());
        assert!(live.begin(&text));
        assert!(source.begin(&text));
        assert_eq!(get(&get(&gate, "gates"), "length").as_f64(), Some(2.0));
        live.protocol.reset().unwrap();
        call(&source.0, "reset", &[]);
        if dispose {
            live.protocol.dispose().unwrap();
            call(&get(&get(&source.0, "h"), "_kittyStorage"), "dispose", &[]);
            call(&get(&source.0, "h"), "dispose", &[]);
            live.registration.dispose();
            call(&get(&source.0, "registration"), "dispose", &[]);
        }
        call(&gate, "restore", &[]);
        call(&get(&gate, "gates"), "forEach", &[eval("f=>f()")]);
        live.finish(true).await;
        source.finish().await;
        same_state(&live, &source);
        same_maps(&live, &source).await;
        assert_eq!(live.added.get(), 1);
    }
}
#[wasm_bindgen_test(async)]
async fn registration_cancellation_and_actual_browser_queue_match_source() {
    let live = Live::new(true);
    let source = Source::new(true);
    live.write("\x1b_Ga=T,i=3,s=6,v=9;AAAA\x18Z").await;
    source.write("\x1b_Ga=T,i=3,s=6,v=9;AAAA\x18Z").await;
    same_state(&live, &source);
    live.begin("\x1b_Ga=T,i=3;");
    source.begin("\x1b_Ga=T,i=3;");
    live.registration.dispose();
    call(&get(&source.0, "registration"), "dispose", &[]);
    live.write("AAAA\x1b\\").await;
    source.write("AAAA\x1b\\").await;
    same_state(&live, &source);
    let token = live.protocol.register().unwrap();
    let registration = js_sys::Function::new_with_args(
        "s",
        "return s.t._core._inputHandler._parser.registerApcHandler({final:'G'},s.h)",
    )
    .call1(&JsValue::UNDEFINED, &source.0)
    .unwrap();
    let text = format!("{}Z", sequence("a=T,i=3,s=6,v=9", &raw()));
    let mut complete = None;
    let pending = js_sys::Promise::new(&mut |resolve, _| complete = Some(resolve));
    live.terminal.write(text.clone().into(), complete).unwrap();
    promise(pending.into()).await;
    source.write(&text).await;
    same_state(&live, &source);
    assert_eq!(live.added.get(), 1);
    token.dispose();
    call(&registration, "dispose", &[]);
}
