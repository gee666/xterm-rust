#![cfg(target_arch = "wasm32")]
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_core::headless::Terminal;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn marker_callback_can_cross_js_and_reset_the_rust_terminal() {
    let terminal = Rc::new(Terminal::new(5, 3, 10).unwrap());
    terminal.write("abcde12345").unwrap();
    let marker = terminal.register_marker(0).unwrap();
    let weak = Rc::downgrade(&terminal);
    let reset = Closure::<dyn FnMut()>::new(move || weak.upgrade().unwrap().reset().unwrap());
    let function: js_sys::Function = reset.as_ref().unchecked_ref::<js_sys::Function>().clone();
    marker.on_dispose().subscribe(move |_| {
        function.call0(&JsValue::UNDEFINED).unwrap();
    });
    terminal.write("\x1b[2Jafter").unwrap();
    let active = terminal.with_state(|state| state.active_buffer()).unwrap();
    let row = active.borrow().lines.get(0).unwrap();
    assert_eq!(
        row.borrow().translate_to_string(true, None, None, None),
        "after".encode_utf16().collect::<Vec<_>>()
    );
    assert!(marker.is_disposed());
    drop(reset);
}

#[wasm_bindgen_test]
fn all_marker_mutating_csi_commands_resume_after_js_reset() {
    for (command, offset) in [
        ("0J", 1),
        ("1J", -1),
        ("2J", 0),
        ("?2J", 0),
        ("3J", -5),
        ("L", 0),
        ("M", 0),
        ("S", -2),
        ("T", 0),
        ("^", 0),
    ] {
        let terminal = Rc::new(Terminal::new(5, 3, 10).unwrap());
        terminal
            .write("0\r\n1\r\n2\r\n3\r\n4\r\n5\r\n6\r\n7")
            .unwrap();
        if command == "0J" {
            terminal.write("\x1b[H").unwrap();
        }
        let old = terminal.with_state(|s| s.active_buffer()).unwrap();
        let marker = terminal.register_marker(offset).unwrap();
        let weak = Rc::downgrade(&terminal);
        let reset = Closure::<dyn FnMut()>::new(move || weak.upgrade().unwrap().reset().unwrap());
        let function: js_sys::Function = reset.as_ref().unchecked_ref::<js_sys::Function>().clone();
        marker.on_dispose().subscribe(move |_| {
            function.call0(&JsValue::UNDEFINED).unwrap();
        });
        terminal.write(&format!("\x1b[{command}")).unwrap();
        assert!(marker.is_disposed(), "{command}");
        let active = terminal.with_state(|s| s.active_buffer()).unwrap();
        assert!(!Rc::ptr_eq(&old, &active), "{command}");
        let b = active.borrow();
        let length = if command.ends_with('J') { 3 } else { 4 };
        assert_eq!(
            (b.x, b.y, b.ybase, b.ydisp, b.lines.len()),
            (0, 0, 0, 0, length),
            "{command}"
        );
        let sparse = matches!(command, "L" | "M");
        for row in 0..length {
            assert_eq!(
                b.lines.get(row).is_none(),
                sparse && row == 3,
                "{command}/{row}"
            );
        }
        drop(reset);
    }
}
