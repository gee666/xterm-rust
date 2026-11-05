//! Shared RenderService fixture, with cancellation programs also run on intact TS.
use super::*;
use xterm_core::core_service::CoreService;

#[wasm_bindgen(module = "/tests/support/render_sync_oracle.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = sourceSyncCancelProgram)]
    fn source_program(program: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = rustSyncCancelProgram)]
    fn rust_program(program: &str, callback: &js_sys::Function) -> Result<JsValue, JsValue>;
}
struct Fixture {
    service: Rc<RenderService>,
    core: Rc<CoreService>,
    log: Log,
}
impl Fixture {
    fn new(host: &JsValue) -> Result<Rc<Self>, JsValue> {
        let (service, terminal) = create_service(host)?;
        let core = terminal.with_state(|s| s.core.clone()).unwrap();
        let log: Log = Rc::default();
        for (name, event) in [
            ("viewport", service.on_rendered_viewport_change()),
            ("render", service.on_render()),
        ] {
            let log = log.clone();
            let _subscription = event.subscribe(move |event| {
                log.borrow_mut().push(format!(
                    "{name}:{}:{}",
                    number(event.start),
                    number(event.end)
                ));
            });
        }
        service.set_renderer(Backend::new("A", &log))?;
        invoke(host, "frame", &[])?;
        log.borrow_mut().clear();
        Ok(Rc::new(Self { service, core, log }))
    }
    fn command(&self, command: &JsValue) -> Result<JsValue, JsValue> {
        let get = |key: &str| js_sys::Reflect::get(command, &key.into());
        match get("op")?.as_string().unwrap().as_str() {
            "mode" => {
                self.core.dec_private_modes.borrow_mut().synchronized_output =
                    get("value")?.as_bool().unwrap();
            }
            "refresh" => self.service.refresh_rows(
                get("start")?.as_f64().unwrap(),
                get("end")?.as_f64().unwrap(),
                true,
                false,
            )?,
            "dispose" => self.service.dispose()?,
            "state" => {
                return Ok(self
                    .core
                    .dec_private_modes
                    .borrow()
                    .synchronized_output
                    .into());
            }
            "drain" => {
                let result = js_sys::Array::new();
                for entry in std::mem::take(&mut *self.log.borrow_mut()) {
                    result.push(&entry.into());
                }
                return Ok(result.into());
            }
            other => panic!("unknown test command {other}"),
        }
        Ok(JsValue::UNDEFINED)
    }
}
fn strings(value: JsValue) -> Vec<String> {
    js_sys::Array::from(&value)
        .iter()
        .map(|value| value.as_string().unwrap())
        .collect()
}
fn check_programs(expected_count: usize, predicate: impl Fn(&str) -> bool) {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/render-sync-cancel.json")).unwrap();
    let mut count = 0;
    for result in fixture["results"].as_array().unwrap() {
        let program = result["program"].as_str().unwrap();
        if !predicate(program) {
            continue;
        }
        count += 1;
        let expected: Vec<_> = result["log"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_owned())
            .collect();
        assert_eq!(
            strings(source_program(program).unwrap()),
            expected,
            "source {program}"
        );
        let slot: Rc<RefCell<Option<Rc<Fixture>>>> = Rc::default();
        let state = slot.clone();
        let callback = Closure::wrap(Box::new(move |host: JsValue, command: JsValue| {
            let current = state.borrow().clone();
            let current = match current {
                Some(current) => current,
                None => {
                    let current = Fixture::new(&host)?;
                    *state.borrow_mut() = Some(current.clone());
                    current
                }
            };
            current.command(&command)
        })
            as Box<dyn Fn(JsValue, JsValue) -> Result<JsValue, JsValue>>);
        let actual = rust_program(program, callback.as_ref().unchecked_ref()).unwrap();
        assert_eq!(strings(actual), expected, "Rust {program}");
        // The JS driver restores patched methods and explicitly disposes before
        // these owners drop, even after a getter or callback throws.
        slot.borrow_mut().take();
    }
    assert_eq!(count, expected_count);
}
#[wasm_bindgen_test]
fn sync_cancel_live_handle_and_reentrant_buffer_flush_replace_dispose() {
    check_programs(10, |name| {
        !name.contains("noncallable")
            && !name.contains("throw")
            && !name.contains("deliver")
            && !name.contains("retained")
    });
}
#[wasm_bindgen_test]
fn sync_cancel_failures_keep_identity_state_and_callable_timer() {
    check_programs(37, |name| {
        name.contains("throw") || name.contains("noncallable")
    });
}
#[wasm_bindgen_test]
fn sync_cancel_timer_delivery_during_getter_or_call_remains_callable() {
    check_programs(7, |name| {
        name.contains("deliver") || name.contains("retained")
    });
}
