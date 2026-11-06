use super::{compare, CASES};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::Terminal;

struct Opened {
    terminal: Terminal,
    parent: web_sys::Element,
}
impl Opened {
    fn new() -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let parent = document.create_element("div").unwrap();
        document.body().unwrap().append_child(&parent).unwrap();
        let terminal = Terminal::new(JsValue::UNDEFINED).unwrap();
        terminal.open(parent.clone().into()).unwrap();
        Self { terminal, parent }
    }
}
impl Drop for Opened {
    fn drop(&mut self) {
        self.terminal.dispose().unwrap();
        self.parent.remove();
    }
}

#[wasm_bindgen_test(async)]
async fn actual_terminal_ime_overlay_bounds_match_unchanged_source() -> Result<(), JsValue> {
    for name in CASES {
        let setup = Opened::new();
        let overlay: web_sys::HtmlElement = setup
            .parent
            .query_selector(".composition-view")
            .unwrap()
            .unwrap()
            .unchecked_into();
        let pending = compare(
            &setup.terminal.textarea().unwrap(),
            &overlay,
            include_str!("../fixtures/composition-geometry-body.js"),
            name,
        )?;
        JsFuture::from(pending).await?;
    }
    Ok(())
}
