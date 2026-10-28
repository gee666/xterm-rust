//! Actual browser composition checks, separate from portable controller doubles.
use super::Viewport;
use crate::{
    terminal::{browser_services::BrowserServices, Terminal},
    XTERM_CSS,
};
use js_sys::{Array, Reflect};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use web_sys::HtmlElement;
use xterm_core::options::OptionValue;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(inline_js = "
export function viewportBrowserOptions() {
  const reads=[];
  const scrollbar={
    get showArrows(){reads.push('showArrows');return false;},
    get showScrollbar(){reads.push('showScrollbar');return true;},
    get width(){reads.push('width');return 18;}
  };
  return {reads,options:{cols:8,rows:3,scrollback:20,smoothScrollDuration:0,scrollbar,
    theme:{background:'#010203',scrollbarSliderBackground:'#112233',
      scrollbarSliderHoverBackground:'#445566',scrollbarSliderActiveBackground:'#778899'}}};
}
")]
extern "C" {
    #[wasm_bindgen(js_name = viewportBrowserOptions)]
    fn browser_options() -> JsValue;
}

struct Fixture {
    terminal: Terminal,
    parent: HtmlElement,
    stylesheet: HtmlElement,
    reads: Array,
}
impl Fixture {
    fn new() -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let stylesheet: HtmlElement = document
            .create_element("style")
            .unwrap()
            .dyn_into()
            .unwrap();
        stylesheet.set_text_content(Some(XTERM_CSS));
        document.body().unwrap().append_child(&stylesheet).unwrap();
        let parent: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
        parent.style().set_property("width", "640px").unwrap();
        parent.style().set_property("height", "240px").unwrap();
        document.body().unwrap().append_child(&parent).unwrap();
        let options = browser_options();
        let reads: Array = Reflect::get(&options, &"reads".into())
            .unwrap()
            .dyn_into()
            .unwrap();
        let terminal = Terminal::new(Reflect::get(&options, &"options".into()).unwrap()).unwrap();
        terminal.open(parent.clone().into()).unwrap();
        Self {
            terminal,
            parent,
            stylesheet,
            reads,
        }
    }
    fn services(&self) -> Rc<BrowserServices> {
        self.terminal
            .shared
            .services
            .borrow()
            .as_ref()
            .unwrap()
            .clone()
    }
    fn viewport(&self) -> Rc<Viewport> {
        self.services().viewport.get().unwrap().clone()
    }
    fn row(&self) -> usize {
        self.terminal
            .shared
            .native
            .with_state(|state| {
                let buffer = state.buffer_service.buffer();
                let row = buffer.borrow().ydisp;
                row
            })
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.terminal.dispose().unwrap();
        self.parent.remove();
        self.stylesheet.remove();
    }
}

#[wasm_bindgen_test]
fn constructor_uses_actual_scroll_element_source_getter_order_and_theme_css() {
    let fixture = Fixture::new();
    let viewport = fixture.viewport();
    let element = viewport.host.element().unwrap();
    let node = element.get_dom_node().unwrap();
    assert_eq!(
        fixture
            .reads
            .to_vec()
            .into_iter()
            .map(|value| value.as_string().unwrap())
            .collect::<Vec<_>>(),
        ["showArrows", "showScrollbar", "showArrows", "width"]
    );
    assert_eq!(
        fixture
            .parent
            .query_selector_all(".xterm-scrollable-element")
            .unwrap()
            .length(),
        1
    );
    assert_eq!(
        node.query_selector_all(".xterm-scrollbar")
            .unwrap()
            .length(),
        2
    );
    assert!(node.query_selector(".xterm-screen").unwrap().is_some());
    assert_eq!(
        viewport
            .root
            .style()
            .get_property_value("background-color")
            .unwrap(),
        "rgb(1, 2, 3)"
    );
    assert_eq!(
        node.style().get_property_value("background-color").unwrap(),
        "rgb(1, 2, 3)"
    );
    let style = viewport.style.borrow().as_ref().unwrap().clone();
    assert_eq!(
        style.text_content().unwrap(),
        concat!(
            ".xterm .xterm-scrollable-element > .xterm-scrollbar > .xterm-slider {\n",
            "  background: #112233;\n}\n",
            ".xterm .xterm-scrollable-element > .xterm-scrollbar > .xterm-slider:hover {\n",
            "  background: #445566;\n}\n",
            ".xterm .xterm-scrollable-element > .xterm-scrollbar > .xterm-slider.xterm-active {\n",
            "  background: #778899;\n}"
        )
    );
    assert!(node.is_connected());
    fixture.terminal.dispose().unwrap();
    assert!(!node.is_connected());
    assert!(!style.is_connected());
}

#[wasm_bindgen_test]
fn actual_element_fractional_pixels_request_real_core_rows_without_feedback() {
    let fixture = Fixture::new();
    fixture
        .terminal
        .shared
        .native
        .write_utf16(
            &"a\r\nb\r\nc\r\nd\r\ne\r\nf"
                .encode_utf16()
                .collect::<Vec<_>>(),
        )
        .unwrap();
    let viewport = fixture.viewport();
    viewport.sync(None).unwrap();
    let cell_height = fixture
        .services()
        .render_service
        .dimensions()
        .unwrap()
        .css
        .cell
        .height;
    assert!(cell_height > 0.0);
    assert_eq!(fixture.row(), 3);
    let requests = Rc::new(RefCell::new(Vec::new()));
    let capture = Rc::clone(&requests);
    let _subscription = viewport
        .on_request_scroll_lines()
        .subscribe(move |row| capture.borrow_mut().push(*row));
    viewport.scroll_to_line(0.0, true).unwrap();
    assert_eq!(fixture.row(), 0);
    viewport.handle_touch_scroll(-cell_height * 1.5).unwrap();
    assert_eq!(fixture.row(), 2);
    assert_eq!(
        viewport.host.element().unwrap().get_scroll_position().top,
        cell_height * 1.5
    );
    viewport.scroll_lines(-1.0).unwrap();
    assert_eq!(fixture.row(), 1);
    assert_eq!(
        viewport.host.element().unwrap().get_scroll_position().top,
        cell_height * 0.5
    );
    assert_eq!(*requests.borrow(), [-3.0, 2.0, -1.0]);
}

#[wasm_bindgen_test]
fn live_options_and_mouse_protocol_update_the_provided_element() {
    let fixture = Fixture::new();
    let viewport = fixture.viewport();
    let element = viewport.host.element().unwrap();
    let options = fixture.terminal.shared.native.options();
    options
        .set("scrollSensitivity", OptionValue::Number(2.5))
        .unwrap();
    options
        .set("fastScrollSensitivity", OptionValue::Number(7.0))
        .unwrap();
    let shared = element.options();
    assert_eq!(
        shared.borrow().mouse_wheel_scroll_sensitivity,
        OptionValue::Number(2.5)
    );
    assert_eq!(
        shared.borrow().fast_scroll_sensitivity,
        OptionValue::Number(7.0)
    );
    let mouse = fixture
        .terminal
        .shared
        .native
        .with_state(|state| Rc::clone(&state.mouse))
        .unwrap();
    mouse.set_active_protocol("VT200").unwrap();
    assert_eq!(shared.borrow().handle_mouse_wheel, OptionValue::Bool(false));
    mouse.set_active_protocol("NONE").unwrap();
    assert_eq!(shared.borrow().handle_mouse_wheel, OptionValue::Bool(true));
}
