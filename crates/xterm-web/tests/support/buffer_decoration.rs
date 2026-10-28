use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlElement;
use xterm_core::{
    buffer::BufferSet,
    decoration::{Decoration, DecorationOptions, DecorationService},
    event::Emitter,
    headless::RenderEvent,
    lifecycle::DisposableHandle,
    options::{OptionValue, OptionsService},
};
use xterm_web::{
    api::{RenderDimensions, Size},
    links::browser_debounce::BrowserRenderDebouncer,
    renderer::buffer_decoration::{
        bind_microtasks, BufferDecorationHost, BufferDecorationRenderer,
    },
};
pub struct Fixture {
    pub renderer: Rc<BufferDecorationRenderer>,
    pub buffers: Rc<BufferSet>,
    pub decorations: Rc<DecorationService>,
    pub dimensions: Rc<Cell<RenderDimensions>>,
    pub viewport: Emitter<RenderEvent>,
    pub changed: Emitter<RenderDimensions>,
    pub dpr: Emitter<f64>,
    screen: HtmlElement,
    scheduler: Rc<BrowserRenderDebouncer>,
    microtasks: DisposableHandle,
}
impl Fixture {
    pub fn new() -> Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let screen: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
        document.body().unwrap().append_child(&screen).unwrap();
        let options = Rc::new(OptionsService::new(
            false,
            std::iter::empty::<(&str, OptionValue)>(),
        ));
        let buffers = Rc::new(BufferSet::new(80, 24, options).unwrap());
        let decorations = Rc::new(DecorationService::with_buffers(&buffers).unwrap());
        let microtasks = bind_microtasks(&decorations).unwrap();
        let mut metrics = RenderDimensions::default();
        metrics.css.cell = Size {
            width: 8.25,
            height: 16.5,
        };
        let dimensions = Rc::new(Cell::new(metrics));
        let scheduler = Rc::new(BrowserRenderDebouncer::try_new(window, |_| Ok(())));
        let weak = Rc::downgrade(&scheduler);
        let current = dimensions.clone();
        let viewport = Emitter::new();
        let changed = Emitter::new();
        let dpr = Emitter::new();
        let renderer = BufferDecorationRenderer::new(BufferDecorationHost {
            document,
            screen: screen.clone(),
            buffers: Rc::clone(&buffers),
            decorations: Rc::clone(&decorations),
            dimensions: Rc::new(move || Ok(current.get())),
            add_refresh_callback: Rc::new(move |callback| {
                weak.upgrade()
                    .ok_or_else(|| js_sys::Error::new("test scheduler dropped"))?
                    .add_try_refresh_callback(move |time| callback(time))
            }),
            viewport_changed: viewport.event(),
            dimensions_changed: changed.event(),
            dpr_changed: dpr.event(),
        })
        .unwrap();
        Self {
            renderer,
            buffers,
            decorations,
            dimensions,
            viewport,
            changed,
            dpr,
            screen,
            scheduler,
            microtasks,
        }
    }
    pub fn register(
        &self,
        line: i64,
        configure: impl FnOnce(&mut DecorationOptions),
    ) -> Decoration {
        let marker = self.buffers.active().borrow_mut().add_marker(line);
        let mut options = DecorationOptions::new(marker);
        configure(&mut options);
        self.decorations
            .register_decoration(Rc::new(RefCell::new(options)))
            .unwrap()
            .unwrap()
    }
    pub fn refresh(&self) {
        self.viewport
            .fire(&RenderEvent { start: 0, end: 23 })
            .unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.renderer.dispose();
        self.microtasks.dispose();
        self.scheduler.dispose().unwrap();
        self.decorations.dispose().unwrap();
        self.screen.remove();
    }
}
pub fn element(decoration: &Decoration) -> HtmlElement {
    decoration
        .element()
        .unwrap()
        .downcast_ref::<HtmlElement>()
        .unwrap()
        .clone()
}
pub fn property(element: &HtmlElement, name: &str) -> String {
    element.style().get_property_value(name).unwrap()
}
pub async fn frame() {
    let window = web_sys::window().unwrap();
    let next = js_sys::Promise::new(&mut |resolve, _| {
        window.request_animation_frame(&resolve).unwrap();
    });
    JsFuture::from(next).await.unwrap();
}
pub async fn microtask() {
    JsFuture::from(js_sys::Promise::resolve(&JsValue::UNDEFINED))
        .await
        .unwrap();
}
