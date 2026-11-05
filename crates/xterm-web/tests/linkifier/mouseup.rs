use super::*;
use js_sys::{Array, Function};

#[wasm_bindgen_test]
fn mouseup_skips_geometry_without_current_link_but_preserves_active_errors() {
    let root = element();
    let calls = Rc::new(Cell::new(0));
    let mode = Rc::new(Cell::new(0));
    let marker: JsValue = js_sys::Object::new().into();
    let count = calls.clone();
    let coordinate_mode = mode.clone();
    let thrown = marker.clone();
    let controller = BrowserLinkifier::with_fallible_providers(
        root.clone(),
        80,
        0,
        Rc::new(move |_| {
            count.set(count.get() + 1);
            match coordinate_mode.get() {
                1 => Ok(None),
                2 => Err(thrown.clone()),
                _ => Ok(Some(Position { x: 5, y: 1 })),
            }
        }),
        Rc::new(|_, _| Ok(())),
        Rc::new(Default::default()),
    )
    .unwrap();
    let provider = Rc::new(Provider::default());
    let _registration = controller.register_provider(provider.clone());
    let log = Rc::new(RefCell::new(Vec::new()));
    let observe: Function =
        Function::new_no_args(include_str!("../../../../scripts/link-mouseup-observe.js"))
            .call0(&JsValue::UNDEFINED)
            .unwrap()
            .dyn_into()
            .unwrap();
    let check = |label: &str, delta, error| {
        let before = calls.get();
        let result = observe
            .call3(
                &JsValue::UNDEFINED,
                root.as_ref(),
                &"mouseup".into(),
                &marker,
            )
            .unwrap();
        let result = Array::from(&result);
        assert_eq!(calls.get() - before, delta, "{label}");
        assert_eq!(result.get(0).as_f64(), Some(f64::from(error)), "{label}");
        assert_eq!(result.get(1).as_bool(), Some(error == 1), "{label}");
    };
    // A throwing geometry callback must remain completely unobserved while idle
    // and while the asynchronous provider has not replied.
    mode.set(2);
    check("idle", 0, 0);
    mode.set(0);
    event(&root, "mousemove");
    mode.set(2);
    check("pending", 0, 0);
    mode.set(0);
    provider.callbacks.borrow_mut().remove(0)(Some(vec![link(log.clone())])).unwrap();
    event(&root, "mousedown");
    check("active", 1, 0);
    assert_eq!(*log.borrow(), ["hover", "activate"]);
    mode.set(1);
    check("invalid position", 1, 0);
    mode.set(2);
    check("active geometry error", 1, 1);
    assert_eq!(*log.borrow(), ["hover", "activate"]);
    event(&root, "mouseleave");
    assert!(controller.current_link().is_none());
    check("cleared", 0, 0);
    controller.dispose().unwrap();
    check("disposed", 0, 0);
    assert_eq!(*log.borrow(), ["hover", "activate", "leave"]);
    root.remove();
}
