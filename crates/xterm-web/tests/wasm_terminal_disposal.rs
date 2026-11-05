#![cfg(target_arch = "wasm32")]

use js_sys::{Object, Reflect};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_web::Terminal;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(inline_js = "
export function disposeFromAddon(term, fail) {
  const store = term._core._store;
  const failure = {};
  let calls = 0;
  let observed = false;
  term._loadAddon(term, {
    activate() {},
    dispose() {
      calls++;
      observed = store._isDisposed && store.isDisposed;
      if (fail) throw failure;
    }
  });
  let thrown;
  try { term.dispose(); } catch (error) { thrown = error; }
  term.dispose();
  return calls === 1 && observed && (fail ? thrown === failure : thrown === undefined);
}
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = disposeFromAddon)]
    fn dispose_from_addon(term: JsValue, fail: bool) -> Result<bool, JsValue>;
}

fn state(store: &JsValue) -> bool {
    let private = Reflect::get(store, &"_isDisposed".into()).unwrap();
    assert_eq!(private, Reflect::get(store, &"isDisposed".into()).unwrap());
    private.as_bool().unwrap()
}

#[wasm_bindgen_test]
fn retained_core_store_reflects_unopened_and_opened_disposal() {
    for opened in [false, true] {
        let terminal = Terminal::new(JsValue::UNDEFINED).unwrap();
        let core = terminal.core_services();
        let store = Reflect::get(&core, &"_store".into()).unwrap();
        assert!(!state(&store));
        assert!(Object::is(
            &store,
            &Reflect::get(&terminal.core_services(), &"_store".into()).unwrap()
        ));
        let parent = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("div")
            .unwrap();
        if opened {
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .body()
                .unwrap()
                .append_child(&parent)
                .unwrap();
            terminal.open(parent.clone().unchecked_into()).unwrap();
            assert!(!state(&store));
        }
        terminal.dispose().unwrap();
        assert!(state(&store));
        terminal.dispose().unwrap();
        assert!(state(&store));
        assert!(Object::is(&core, &terminal.core_services()));
        assert!(!parent.has_child_nodes());
        parent.remove();
    }
}

#[wasm_bindgen_test]
fn store_is_disposed_before_addon_callbacks_even_when_they_throw() {
    for fail in [false, true] {
        let terminal = Terminal::new(JsValue::UNDEFINED).unwrap();
        assert!(dispose_from_addon(terminal.into(), fail).unwrap());
    }
}

#[wasm_bindgen_test]
fn internal_core_disposal_and_public_disposal_share_one_flag() {
    let terminal = Terminal::new(JsValue::UNDEFINED).unwrap();
    let store = Reflect::get(&terminal.core_services(), &"_store".into()).unwrap();
    assert!(!state(&store));
    terminal.dispose_core().unwrap();
    assert!(state(&store));
    terminal.dispose().unwrap();
    assert!(state(&store));
}
