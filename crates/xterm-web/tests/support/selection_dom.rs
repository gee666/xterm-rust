#![cfg(target_arch = "wasm32")]
// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use js_sys::{Function, Promise, Reflect};
use std::{
    cell::{Cell, RefCell},
    num::NonZeroU32,
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Event, HtmlElement};
use xterm_core::{
    buffer::BufferService,
    event::{Emitter, Event as CoreEvent},
    options::{OptionValue, OptionsService},
};
use xterm_web::{
    dom::{BrowserDom, DomOptions},
    input::mouse_coords::MouseGeometry,
    selection::{
        dom::{LinkSelection, SelectionDom, SelectionDomOptions, SelectionHost},
        model::{Dimensions, Position},
        pointer::SelectionMouseOptions,
        service::{RedrawRequest, SelectionPlatform, SelectionService},
    },
};

pub(super) struct Host {
    pub(super) buffers: Rc<BufferService>,
    pub(super) options: RefCell<SelectionDomOptions>,
    pub(super) link: Cell<Option<LinkSelection>>,
    pub(super) user_input: Emitter<()>,
    pub(super) redraws: RefCell<Vec<RedrawRequest>>,
    pub(super) changes: Cell<u32>,
    pub(super) linux: RefCell<Vec<Vec<u16>>>,
    input: RefCell<Vec<Vec<u16>>>,
    pub(super) cursor_moves: RefCell<Vec<(Position, bool)>>,
    errors: RefCell<Vec<JsValue>>,
    pub(super) next_scroll: RefCell<Option<Function>>,
}
impl SelectionHost for Host {
    fn options(&self) -> Result<SelectionDomOptions, JsValue> {
        Ok(self.options.borrow().clone())
    }
    fn current_link(&self) -> Result<Option<LinkSelection>, JsValue> {
        Ok(self.link.get())
    }
    fn on_user_input(&self) -> Result<CoreEvent<()>, JsValue> {
        Ok(self.user_input.event())
    }
    fn scroll_lines(&self, amount: i64, suppress: bool) -> Result<(), JsValue> {
        self.buffers.scroll_lines(amount, suppress).unwrap();
        if let Some(resolve) = self.next_scroll.borrow_mut().take() {
            resolve.call0(&JsValue::NULL)?;
        }
        Ok(())
    }
    fn data(&self, data: &[u16], _: bool) -> Result<(), JsValue> {
        self.input.borrow_mut().push(data.to_vec());
        Ok(())
    }
    fn move_cursor(&self, position: Position, application: bool) -> Result<(), JsValue> {
        self.cursor_moves.borrow_mut().push((position, application));
        Ok(())
    }
    fn redraw(&self, request: RedrawRequest) -> Result<(), JsValue> {
        self.redraws.borrow_mut().push(request);
        Ok(())
    }
    fn selection_changed(&self) -> Result<(), JsValue> {
        self.changes.set(self.changes.get() + 1);
        Ok(())
    }
    fn linux_mouse_selection(&self, text: &[u16]) -> Result<(), JsValue> {
        self.linux.borrow_mut().push(text.to_vec());
        Ok(())
    }
    fn report_error(&self, error: JsValue) {
        self.errors.borrow_mut().push(error);
    }
}
pub(super) struct Setup {
    pub(super) parent: HtmlElement,
    pub(super) dom: BrowserDom,
    pub(super) host: Rc<Host>,
    pub(super) selection: Rc<SelectionService>,
    pub(super) bindings: SelectionDom,
}
impl Setup {
    pub(super) fn new() -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let parent: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
        document.body().unwrap().append_child(&parent).unwrap();
        let dom = BrowserDom::new(&parent, &DomOptions::default()).unwrap();
        dom.screen.style().set_property("padding", "0px").unwrap();
        let options = Rc::new(OptionsService::new(
            false,
            [
                ("cols", OptionValue::Number(10.0)),
                ("rows", OptionValue::Number(2.0)),
            ],
        ));
        let buffers = Rc::new(BufferService::new(options).unwrap());
        {
            let active = buffers.buffer();
            let mut buffer = active.borrow_mut();
            buffer.fill_viewport_rows(None).unwrap();
            for row in 0..2 {
                let line = buffer.lines.get(row).unwrap();
                let mut line = line.borrow_mut();
                for (column, character) in "hello rust".chars().enumerate() {
                    line.set_cell_from_codepoint(
                        column,
                        u32::from(character),
                        1,
                        &Default::default(),
                    );
                }
            }
        }
        let selection = Rc::new(SelectionService::new(SelectionPlatform {
            linux: true,
            ..SelectionPlatform::default()
        }));
        let host = Rc::new(Host {
            buffers: buffers.clone(),
            options: RefCell::new(SelectionDomOptions {
                platform: SelectionPlatform {
                    linux: true,
                    ..SelectionPlatform::default()
                },
                mouse: SelectionMouseOptions::default(),
                word_separator: " ".encode_utf16().collect(),
                alt_click_moves_cursor: true,
                application_cursor_keys: true,
            }),
            link: Cell::new(None),
            user_input: Emitter::new(),
            redraws: RefCell::new(Vec::new()),
            changes: Cell::new(0),
            linux: RefCell::new(Vec::new()),
            input: RefCell::new(Vec::new()),
            cursor_moves: RefCell::new(Vec::new()),
            errors: RefCell::new(Vec::new()),
            next_scroll: RefCell::new(None),
        });
        let bindings = SelectionDom::new(
            &dom,
            buffers,
            selection.clone(),
            Rc::new(|| {
                Ok(MouseGeometry {
                    rect_left: 0.0,
                    rect_top: 0.0,
                    padding_left: 0.0,
                    padding_top: 0.0,
                    valid_char_size: true,
                    cell_width: 10.0,
                    cell_height: 20.0,
                    canvas_width: 100.0,
                    canvas_height: 40.0,
                })
            }),
            host.clone(),
        )
        .unwrap();
        Self {
            parent,
            dom,
            host,
            selection,
            bindings,
        }
    }
    pub(super) fn dimensions(&self) -> Dimensions {
        Dimensions {
            columns: NonZeroU32::new(10).unwrap(),
            rows: NonZeroU32::new(2).unwrap(),
            scrollback: 0,
        }
    }
    pub(super) fn event(
        &self,
        name: &str,
        column: i32,
        row: i32,
        detail: i32,
        alt: bool,
        shift: bool,
    ) -> Event {
        let rect = self.dom.screen.get_bounding_client_rect();
        let init = js_sys::Object::new();
        for (key, value) in [
            ("bubbles", true.into()),
            ("cancelable", true.into()),
            ("clientX", (rect.left() + f64::from(column * 10 + 1)).into()),
            ("clientY", (rect.top() + f64::from(row * 20 + 1)).into()),
            ("button", 0.into()),
            ("detail", detail.into()),
            ("altKey", alt.into()),
            ("shiftKey", shift.into()),
        ] {
            Reflect::set(&init, &key.into(), &value).unwrap();
        }
        let constructor = Reflect::get(&js_sys::global(), &"MouseEvent".into())
            .unwrap()
            .dyn_into::<Function>()
            .unwrap();
        Reflect::construct(&constructor, &js_sys::Array::of2(&name.into(), &init))
            .unwrap()
            .dyn_into()
            .unwrap()
    }
    pub(super) fn dispatch(
        &self,
        name: &str,
        column: i32,
        row: i32,
        detail: i32,
        alt: bool,
        shift: bool,
    ) -> Event {
        let event = self.event(name, column, row, detail, alt, shift);
        if name == "mousedown" {
            self.dom.element.dispatch_event(&event).unwrap();
        } else {
            self.dom.document.dispatch_event(&event).unwrap();
        }
        event
    }
}
impl Drop for Setup {
    fn drop(&mut self) {
        self.bindings.dispose().unwrap();
        assert!(
            self.host.errors.borrow().is_empty(),
            "{:?}",
            self.host.errors.borrow()
        );
        self.parent.remove();
    }
}
pub(super) async fn frame() {
    let promise = Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .request_animation_frame(resolve.unchecked_ref())
            .unwrap();
    });
    JsFuture::from(promise).await.unwrap();
}
