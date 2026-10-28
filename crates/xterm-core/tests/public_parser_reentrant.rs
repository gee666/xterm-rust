use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::{
    headless::Terminal,
    lifecycle::DisposableHandle,
    parser::{
        escape::{EscapeSequenceParser, FunctionIdentifier, RegistrarError},
        handler::HandlerResult,
    },
    public_api::{ParserApi, ParserRegistrations},
};

fn register(
    api: &mut ParserApi<'_>,
    family: usize,
    mut callback: impl FnMut() -> HandlerResult + 'static,
) -> Result<DisposableHandle, RegistrarError> {
    let id = FunctionIdentifier::new("Z");
    match family {
        0 => api.register_csi_handler(id, move |_| callback()),
        1 => api.register_dcs_handler(id, move |_, _| callback()),
        2 => api.register_esc_handler(id, callback),
        3 => api.register_osc_handler(666.0, move |_| callback()),
        4 => api.register_apc_handler(id, move |_| callback()),
        _ => unreachable!(),
    }
}

#[test]
fn all_five_public_callbacks_can_register_without_borrowing_active_engine() {
    let sequences = [
        "\x1b[Z",
        "\x1bPZpayload\x1b\\",
        "\x1bZ",
        "\x1b]666;payload\x07",
        "\x1b_Zpayload\x1b\\",
    ];
    for (family, sequence) in sequences.iter().enumerate() {
        let terminal = Rc::new(Terminal::new(20, 3, 10).unwrap());
        let log = Rc::new(RefCell::new(Vec::new()));
        let weak = Rc::downgrade(&terminal);
        let captured = log.clone();
        let installed = Cell::new(false);
        terminal
            .with_parser(|api| {
                register(api, family, move || {
                    captured.borrow_mut().push("first");
                    if !installed.replace(true) {
                        let next = captured.clone();
                        weak.upgrade()
                            .unwrap()
                            .with_parser(|api| {
                                register(api, family, move || {
                                    next.borrow_mut().push("second");
                                    HandlerResult::Ready(true)
                                })
                            })
                            .unwrap()
                            .unwrap();
                    }
                    HandlerResult::Ready(false)
                })
            })
            .unwrap()
            .unwrap();
        terminal.write(sequence).unwrap();
        assert_eq!(*log.borrow(), ["first"], "current dispatch family{family}");
        terminal.write(sequence).unwrap();
        assert_eq!(
            *log.borrow(),
            ["first", "second"],
            "next dispatch family{family}"
        );
    }
}

#[test]
fn callbacks_can_dispose_their_own_public_token_in_every_family() {
    let sequences = [
        "\x1b[Z",
        "\x1bPZpayload\x1b\\",
        "\x1bZ",
        "\x1b]666;payload\x07",
        "\x1b_Zpayload\x1b\\",
    ];
    for (family, sequence) in sequences.iter().enumerate() {
        let terminal = Terminal::new(20, 3, 10).unwrap();
        let slot: Rc<RefCell<Option<DisposableHandle>>> = Rc::default();
        let captured = slot.clone();
        let calls = Rc::new(Cell::new(0));
        let count = calls.clone();
        let handle = terminal
            .with_parser(|api| {
                register(api, family, move || {
                    count.set(count.get() + 1);
                    captured.borrow().as_ref().unwrap().dispose();
                    HandlerResult::Ready(true)
                })
            })
            .unwrap()
            .unwrap();
        *slot.borrow_mut() = Some(handle.clone());
        terminal.write(sequence).unwrap();
        terminal.write(sequence).unwrap();
        handle.dispose();
        assert_eq!(calls.get(), 1, "family{family}");
    }
}

#[test]
fn dropped_parser_bundle_reports_error_instead_of_a_successful_registration() {
    let registrations = {
        let parser = EscapeSequenceParser::new();
        ParserRegistrations::new(&parser)
    };
    let mut api = ParserApi::from_registrations(&registrations);
    for family in 0..5 {
        assert_eq!(
            register(&mut api, family, || HandlerResult::Ready(true)).err(),
            Some(RegistrarError::ParserDropped)
        );
    }
    assert!(registrations.osc_registrar().clear_handler(666.0).is_none());
    assert!(registrations
        .esc_registrar()
        .clear(FunctionIdentifier::new("Z"))
        .is_err());
}
