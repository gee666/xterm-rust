#![cfg(target_arch = "wasm32")]
#[path = "image_addon_support/constructor_order.rs"]
mod constructor_order;
#[path = "image_addon_support/handlers.rs"]
mod handlers;
#[path = "image_addon_support/inactive_coordinates.rs"]
mod inactive_coordinates;
#[path = "image_addon_support/inspection.rs"]
mod inspection;
#[path = "image_addon_support/inspection_activation.rs"]
mod inspection_activation;
#[path = "image_addon_support/options.rs"]
mod options;
#[path = "image_addon_support/raw_flags.rs"]
mod raw_flags;
#[path = "image_addon_support/raw_iip_limit.rs"]
mod raw_iip_limit;
#[path = "image_addon_support/raw_kitty_limit.rs"]
mod raw_kitty_limit;
#[path = "image_addon_support/raw_palette.rs"]
mod raw_palette;
#[path = "image_addon_support/raw_palette_failures.rs"]
mod raw_palette_failures;
#[path = "image_addon_support/raw_palette_primitives.rs"]
mod raw_palette_primitives;
#[path = "image_addon_support/raw_palette_release.rs"]
mod raw_palette_release;
#[path = "image_addon_support/raw_placeholder.rs"]
mod raw_placeholder;
#[path = "image_addon_support/raw_sixel_limit.rs"]
mod raw_sixel_limit;
#[path = "image_addon_support/raw_storage_limit.rs"]
mod raw_storage_limit;
#[path = "image_addon_support/render_dimensions.rs"]
mod render_dimensions;
#[path = "image_addon_support/row_coordinates.rs"]
mod row_coordinates;
#[path = "image_addon_support/mod.rs"]
mod support;
#[path = "image_addon_support/visibility.rs"]
mod visibility;
use support::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_web::image::ImageAddon;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn inactive_properties_and_constructor_copy_match_source() {
    let rust = ImageAddon::new(JsValue::UNDEFINED).unwrap();
    let source = eval("new SourceImageAddon()");
    assert!(rust.storage_inspection().is_undefined());
    assert_eq!(
        rust.storage_limit(),
        get(&source, "storageLimit").as_f64().unwrap()
    );
    assert_eq!(
        rust.storage_usage().unwrap(),
        get(&source, "storageUsage").as_f64().unwrap()
    );
    assert_eq!(rust.show_placeholder(), get(&source, "showPlaceholder"));
    assert!(rust
        .get_image_at_buffer_cell(0.0, 0.0)
        .unwrap()
        .is_undefined());
    rust.set_storage_limit(64.0.into()).unwrap();
    rust.set_show_placeholder(JsValue::FALSE).unwrap();
    assert_eq!(rust.storage_limit(), -1.0);
    assert_eq!(rust.show_placeholder(), JsValue::FALSE);
    assert!(!rust.reset().unwrap());
    assert_eq!(rust.on_image_added(), rust.on_image_added());
    rust.dispose().unwrap();
    call(&source, "dispose", &[]);
}
#[wasm_bindgen_test(async)]
async fn all_protocols_share_public_storage_events_and_pixels() {
    let pair = Pair::new(true, "({showPlaceholder:false})");
    pair.ready().await;
    for sequence in [sixel(), "\x1b[3;1H", iip(), "\x1b[5;1H", kitty()] {
        pair.write(sequence).await;
        pair.compare();
    }
    assert_eq!(get(&pair.events, "count"), JsValue::from(3));
    pair.compare_image(0.0, 0.0);
    pair.compare_image(0.0, 2.0);
    pair.compare_image(0.0, 4.0);
}
#[wasm_bindgen_test(async)]
async fn graphics_queries_scrolling_limits_and_reset_match_source() {
    let pair = Pair::new(true, "({showPlaceholder:false,sixelPaletteLimit:256})");
    pair.ready().await;
    for text in [
        "\x1b[c",
        "\x1b[1c",
        "\x1b[?1;1S",
        "\x1b[?1;3;32S",
        "\x1b[?1;4S",
        "\x1b[?2;1S",
        "\x1b[?2;4S",
        "\x1b[?3;1S",
        "\x1b[?1;9S",
        "\x1b[?1;3;4097S",
        "\x1b[?1;3;1:2S",
        "\x1b[?80h",
        sixel(),
        "\x1b[?80l",
        "\r\n",
        sixel(),
        "\x1b[?1;2S",
        "\x1b[!p",
    ] {
        pair.write(text).await;
        pair.compare();
    }
    assert_eq!(pair.addon.storage_usage().unwrap(), 0.0);
    for limit in [64.0, 0.0, 16.0] {
        let rust = pair.addon.set_storage_limit(limit.into());
        let source = js_sys::Reflect::set(
            &get(&pair.source, "addon"),
            &"storageLimit".into(),
            &limit.into(),
        );
        assert_eq!(rust.is_ok(), source.is_ok());
        pair.compare();
    }
    pair.addon.set_show_placeholder(JsValue::TRUE).unwrap();
    js_sys::Reflect::set(
        &get(&pair.source, "addon"),
        &"showPlaceholder".into(),
        &true.into(),
    )
    .unwrap();
    pair.compare();
    pair.write("\x1bc").await;
    pair.compare();
}
#[wasm_bindgen_test(async)]
async fn support_flags_disabled_before_open_and_window_reports_match_source() {
    let pair=Pair::new(false,"({sixelSupport:false,iipSupport:false,kittySupport:false,enableSizeReports:false,showPlaceholder:false})");
    pair.write("\x1b[c\x1b[?1;1S\x1b[?2;1S").await;
    pair.compare();
    for text in [sixel(), iip(), kitty()] {
        pair.write(text).await;
        pair.compare();
    }
    assert_eq!(pair.addon.storage_usage().unwrap(), 0.0);
    assert_eq!(get(&pair.events, "count"), JsValue::from(0));
    let enabled = Pair::new(true, "({showPlaceholder:false})");
    enabled.ready().await;
    let actual = get(&enabled.terminal.options(), "windowOptions");
    let source = get(&get(&enabled.source, "terminal"), "options");
    let expected = get(&source, "windowOptions");
    for key in ["getWinSizePixels", "getCellSizePixels", "getWinSizeChars"] {
        assert_eq!(get(&actual, key), get(&expected, key));
    }
    enabled.addon.dispose().unwrap();
    assert_eq!(get(&actual, "getWinSizePixels"), JsValue::TRUE);
}
#[wasm_bindgen_test(async)]
async fn alternate_resize_public_reset_and_disposal_match_source() {
    let pair = Pair::new(true, "({showPlaceholder:false})");
    pair.ready().await;
    pair.write(sixel()).await;
    pair.compare();
    pair.terminal.resize(24.0, 12.0).unwrap();
    call(
        &get(&pair.source, "terminal"),
        "resize",
        &[24.into(), 12.into()],
    );
    pair.frame().await;
    pair.compare();
    pair.write("\x1b[?1049h").await;
    pair.write(sixel()).await;
    pair.compare();
    pair.write("\x1b[?1049l").await;
    pair.compare();
    pair.terminal.reset().unwrap();
    call(&get(&pair.source, "terminal"), "reset", &[]);
    pair.compare();
    pair.write(sixel()).await;
    pair.compare();
    pair.addon.dispose().unwrap();
    call(&get(&pair.source, "addon"), "dispose", &[]);
    pair.compare();
    pair.write(sixel()).await;
    pair.compare();
    assert_eq!(pair.addon.storage_usage().unwrap(), 0.0);
}

#[wasm_bindgen_test(async)]
async fn shipped_public_constructor_and_terminal_addon_manager_are_connected() {
    public_bootstrap().await;
}

#[wasm_bindgen_test(async)]
async fn image_added_can_reenter_reset_without_borrowing_native_state() {
    use std::rc::Rc;
    use wasm_bindgen::{closure::Closure, JsCast};
    let pair = Rc::new(Pair::new(true, "({showPlaceholder:false})"));
    pair.ready().await;
    let weak = Rc::downgrade(&pair);
    let callback = Closure::wrap(Box::new(move || {
        assert!(!weak.upgrade().unwrap().addon.reset().unwrap());
    }) as Box<dyn FnMut()>);
    let registration = pair
        .addon
        .on_image_added()
        .call1(&JsValue::UNDEFINED, callback.as_ref().unchecked_ref())
        .unwrap();
    call(
        &get(&pair.source, "addon"),
        "onImageAdded",
        &[js_sys::Function::new_with_args("", "this.reset()")
            .bind0(&get(&pair.source, "addon"))
            .into()],
    );
    for text in [sixel(), iip(), kitty()] {
        pair.write(text).await;
        pair.compare();
        assert_eq!(pair.addon.storage_usage().unwrap(), 0.0);
    }
    call(&registration, "dispose", &[]);
}

#[wasm_bindgen_test(async)]
async fn repeated_activation_keeps_old_protocol_and_current_parent_contexts() {
    use std::{cell::RefCell, rc::Rc};
    use xterm_web::Terminal;
    let pair = Pair::new(true, "({showPlaceholder:false})");
    pair.ready().await;
    let second = Terminal::new(eval(
        "({cols:40,rows:12,allowProposedApi:true,logLevel:'off'})",
    ))
    .unwrap();
    let original =
        eval("new SourceTerminal({cols:40,rows:12,allowProposedApi:true,logLevel:'off'})");
    pair.addon.activate(&second).unwrap();
    call(
        &get(&pair.source, "addon"),
        "activate",
        std::slice::from_ref(&original),
    );
    pair.ready().await;
    pair.write(sixel()).await;
    pair.compare();
    assert_eq!(pair.addon.storage_usage().unwrap(), 0.0);
    let replies = Rc::new(RefCell::new(Vec::<Vec<u16>>::new()));
    let capture = replies.clone();
    let token = second
        .shared_native()
        .on_data()
        .subscribe(move |text| capture.borrow_mut().push(text.clone()));
    let source_replies = eval("[]");
    call(
        &original,
        "onData",
        &[get(&source_replies, "push")
            .unchecked_into::<js_sys::Function>()
            .bind0(&source_replies)
            .into()],
    );
    pair.write("\x1b[c\x1b[?2;1S").await;
    pair.compare();
    assert_eq!(replies.borrow().len(), 2);
    let actual: js_sys::Array = replies
        .borrow()
        .iter()
        .map(|text| JsValue::from(String::from_utf16(text).unwrap()))
        .collect();
    assert_eq!(
        js_sys::JSON::stringify(&actual).unwrap(),
        js_sys::JSON::stringify(&source_replies).unwrap()
    );
    token.dispose();
    pair.addon.dispose().unwrap();
    call(&get(&pair.source, "addon"), "dispose", &[]);
    second.dispose().unwrap();
    call(&original, "dispose", &[]);
}

#[wasm_bindgen_test(async)]
async fn pending_bitmap_finishes_after_coordinator_disposal_without_new_events() {
    late_completion().await;
}

#[wasm_bindgen_test(async)]
async fn configured_protocol_size_and_pixel_limits_reach_actual_producers() {
    for options in [
        "({showPlaceholder:false,sixelSizeLimit:1,iipSizeLimit:8,kittySizeLimit:1})",
        "({showPlaceholder:false,sixelSupport:false,pixelLimit:1})",
        "({showPlaceholder:false,iipSizeLimit:41.5})",
    ] {
        let pair = Pair::new(true, options);
        pair.ready().await;
        for text in [sixel(), iip(), kitty()] {
            pair.write(text).await;
            pair.compare();
        }
    }
}

#[wasm_bindgen_test(async)]
async fn throwing_renderer_disposal_preserves_tokens_storage_and_event_emitter() {
    failed_disposal().await;
}

#[wasm_bindgen_test(async)]
async fn constructor_getters_are_copied_twice_and_reset_uses_second_defaults() {
    let pair=Pair::new(true,"(()=>{let count=0;return {showPlaceholder:false,get sixelPaletteLimit(){return ++count*128}}})()");
    pair.ready().await;
    pair.write("\x1b[?1;1S\x1b[?1;2S\x1b[?1;1S").await;
    pair.compare();
    assert_eq!(pair.replies.borrow().len(), 3);
    assert_eq!(
        String::from_utf16(&pair.replies.borrow()[0]).unwrap(),
        "\x1b[?1;0;128S"
    );
    assert_eq!(
        String::from_utf16(&pair.replies.borrow()[1]).unwrap(),
        "\x1b[?1;0;256S"
    );
}
