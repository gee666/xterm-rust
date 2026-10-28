// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. DOM component fixtures, distinct from Terminal E2E.
pub mod clock;
pub mod errors;
mod text;
use clock::Clock;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
pub use text::{attribute, text, units};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, HtmlElement};
use xterm_core::{
    event::{EmitError, Emitter},
    lifecycle::DisposableHandle,
};
use xterm_web::accessibility::browser::Listener;
use xterm_web::accessibility::{
    AccessibilityEvent, AccessibilityHost, AccessibilityManager, Dimensions, LocalizableStrings,
    Row, SelectionRange,
};

use xterm_wasm_support::js_error::{to_js_error, JsCallbackError};
pub struct Host {
    pub rows: Cell<usize>,
    pub cols: usize,
    pub offset: Cell<usize>,
    pub lines: RefCell<Vec<Row>>,
    pub dimensions: Cell<Dimensions>,
    pub events: Emitter<AccessibilityEvent>,
    pub selections: RefCell<Vec<SelectionRange>>,
    pub clears: Cell<usize>,
    pub scrolls: RefCell<Vec<i32>>,
}
impl Host {
    pub fn new() -> Rc<Self> {
        let normal = |value: &str| {
            let text = units(value);
            Row {
                columns: (0..=text.len()).collect(),
                text,
            }
        };
        Rc::new(Self {
            rows: Cell::new(3),
            cols: 8,
            offset: Cell::new(2),
            lines: RefCell::new(vec![
                normal("old0"),
                normal("old1"),
                Row {
                    text: units("A😀字"),
                    columns: vec![0, 1, 1, 3, 5],
                },
                normal(""),
                normal("tail"),
                normal("last"),
            ]),
            dimensions: Cell::new(Dimensions {
                cell_width: 8.0,
                cell_height: 16.0,
                canvas_width: 64.0,
                font_size: 12.0,
            }),
            events: Emitter::new(),
            selections: RefCell::new(Vec::new()),
            clears: Cell::new(0),
            scrolls: RefCell::new(Vec::new()),
        })
    }
    pub fn emit(&self, event: AccessibilityEvent) -> Result<(), JsValue> {
        self.events
            .fire(&event)
            .map_err(|error| to_js_error(&error))
    }
    pub fn render(&self) -> Result<(), JsValue> {
        self.emit(AccessibilityEvent::Render {
            start: 0.0,
            end: self.rows.get() as f64 - 1.0,
        })
    }
}
impl AccessibilityHost for Host {
    fn rows(&self) -> Result<usize, JsValue> {
        Ok(self.rows.get())
    }
    fn cols(&self) -> Result<usize, JsValue> {
        Ok(self.cols)
    }
    fn display_offset(&self) -> Result<usize, JsValue> {
        Ok(self.offset.get())
    }
    fn buffer_length(&self) -> Result<usize, JsValue> {
        Ok(self.lines.borrow().len())
    }
    fn row(&self, row: usize) -> Result<Option<Row>, JsValue> {
        Ok(self.lines.borrow().get(row).cloned())
    }
    fn dimensions(&self) -> Result<Dimensions, JsValue> {
        Ok(self.dimensions.get())
    }
    fn scroll_lines(&self, amount: i32) -> Result<(), JsValue> {
        self.scrolls.borrow_mut().push(amount);
        let max = self.lines.borrow().len().saturating_sub(self.rows.get());
        let next = (self.offset.get() as i64 + i64::from(amount)).clamp(0, max as i64);
        self.offset.set(next as usize);
        self.emit(AccessibilityEvent::Scroll)
    }
    fn clear_selection(&self) -> Result<(), JsValue> {
        self.clears.set(self.clears.get() + 1);
        Ok(())
    }
    fn select(&self, column: f64, row: f64, length: f64) -> Result<(), JsValue> {
        self.selections.borrow_mut().push(SelectionRange {
            row,
            column,
            length,
        });
        Ok(())
    }
    fn subscribe(&self, listener: Listener) -> Result<Vec<DisposableHandle>, JsValue> {
        Ok(vec![self.events.event().try_subscribe(move |event| {
            listener(event.clone()).map_err(|error| EmitError::callback(JsCallbackError(error)))
        })])
    }
}
pub struct Fixture {
    pub manager: AccessibilityManager,
    pub host: Rc<Host>,
    pub labels: LocalizableStrings,
    pub document: Document,
    pub root: HtmlElement,
    pub before: HtmlElement,
    pub after: HtmlElement,
    style: HtmlElement,
    pub clock: Clock,
}
impl Fixture {
    pub fn new() -> Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let clock = Clock::new(&window);
        let style: HtmlElement = document
            .create_element("style")
            .unwrap()
            .dyn_into()
            .unwrap();
        style.set_text_content(Some(xterm_web::XTERM_CSS));
        document.body().unwrap().append_child(&style).unwrap();
        let before = outside(&document, "before");
        let root: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
        root.set_class_name("xterm");
        root.style().set_property("height", "48px").unwrap();
        let existing = document.create_element("span").unwrap();
        existing.set_class_name("existing");
        root.append_child(&existing).unwrap();
        document.body().unwrap().append_child(&root).unwrap();
        let after = outside(&document, "after");
        let host = Host::new();
        let labels = LocalizableStrings::new();
        let manager = AccessibilityManager::new(
            window,
            document.clone(),
            Some(&root),
            host.clone(),
            labels.clone(),
        )
        .unwrap();
        Self {
            manager,
            host,
            labels,
            document,
            root,
            before,
            after,
            style,
            clock,
        }
    }
    pub fn tree(&self) -> HtmlElement {
        self.manager
            .container()
            .query_selector("[role=list]")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap()
    }
    pub fn row(&self, index: u32) -> HtmlElement {
        self.tree()
            .children()
            .item(index)
            .unwrap()
            .dyn_into()
            .unwrap()
    }
    pub fn live(&self) -> HtmlElement {
        self.manager
            .container()
            .query_selector(".live-region")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap()
    }
    pub fn selection_change(&self) -> Result<(), JsValue> {
        self.document
            .dispatch_event(&web_sys::Event::new("selectionchange").unwrap())
            .map(|_| ())
    }
    pub fn render(&self) {
        self.host.render().unwrap();
        self.clock.advance(1000.0).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.manager.dispose().unwrap();
        self.root.remove();
        self.before.remove();
        self.after.remove();
        self.style.remove();
        self.document
            .get_selection()
            .unwrap()
            .unwrap()
            .remove_all_ranges()
            .unwrap();
    }
}
fn outside(document: &Document, value: &str) -> HtmlElement {
    let element: HtmlElement = document.create_element("p").unwrap().dyn_into().unwrap();
    element.set_text_content(Some(value));
    element.set_tab_index(0);
    document.body().unwrap().append_child(&element).unwrap();
    element
}
