use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::{
    headless::{HeadlessError, Terminal},
    parser::{escape::FunctionIdentifier, handler::HandlerResult},
    public_api::{BufferType, TerminalAddon},
};

struct Probe {
    activated_cols: Cell<usize>,
    disposed: Cell<usize>,
}
impl TerminalAddon<Terminal> for Probe {
    type Error = HeadlessError;
    fn activate(&self, terminal: &Terminal) -> Result<(), Self::Error> {
        self.activated_cols.set(terminal.cols()?);
        Ok(())
    }
    fn dispose(&self) -> Result<(), Self::Error> {
        self.disposed.set(self.disposed.get() + 1);
        Ok(())
    }
}

#[test]
fn source_load_addon_assertions_use_actual_public_terminal() {
    let terminal = Terminal::new(5, 3, 10).unwrap();
    let addon = Rc::new(Probe {
        activated_cols: Cell::new(0),
        disposed: Cell::new(0),
    });
    let handle = terminal.load_addon(addon.clone()).unwrap();
    assert_eq!(addon.activated_cols.get(), 5);
    assert_eq!(addon.disposed.get(), 0);
    handle.dispose().unwrap();
    assert_eq!(addon.disposed.get(), 1);
    let second = Rc::new(Probe {
        activated_cols: Cell::new(0),
        disposed: Cell::new(0),
    });
    terminal.load_addon(second.clone()).unwrap();
    assert_eq!(second.disposed.get(), 0);
    terminal.dispose().unwrap();
    assert_eq!(second.disposed.get(), 1);
    assert_eq!(addon.disposed.get(), 1);
}

#[test]
fn buffer_namespace_survives_actual_writes_alt_switch_and_reset() {
    let terminal = Terminal::new(5, 3, 10).unwrap();
    let namespace = terminal.buffer().unwrap();
    assert!(Rc::ptr_eq(&namespace, &terminal.buffer().unwrap()));
    let normal = namespace.normal();
    terminal.write("abc").unwrap();
    assert_eq!(normal.cursor_x().unwrap(), 3);
    let line = normal.get_line(0).unwrap().unwrap();
    assert_eq!(
        line.borrow().translate_to_string(true, None, None, None),
        "abc".encode_utf16().collect::<Vec<_>>()
    );
    let events = Rc::new(RefCell::new(Vec::new()));
    let captured = events.clone();
    namespace
        .on_buffer_change()
        .subscribe(move |view| captured.borrow_mut().push(view.buffer_type()));
    terminal.write("\x1b[?1049h").unwrap();
    assert_eq!(
        namespace.active().unwrap().buffer_type(),
        BufferType::Alternate
    );
    terminal.write("\x1b[?1049l").unwrap();
    assert!(Rc::ptr_eq(&normal, &namespace.active().unwrap()));
    terminal.reset().unwrap();
    assert!(Rc::ptr_eq(&normal, &namespace.active().unwrap()));
    assert_eq!(
        *events.borrow(),
        [
            BufferType::Alternate,
            BufferType::Normal,
            BufferType::Normal
        ]
    );
}

#[test]
fn every_public_parser_family_works_through_utf8_and_utf16_terminal_chunks() {
    let inputs = [
        "\x1b[Z",
        "\x1bPZdata\x1b\\",
        "\x1bZ",
        "\x1b]666;data\x07",
        "\x1b_Zdata\x1b\\",
    ];
    for (family, sequence) in inputs.iter().enumerate() {
        for bytes in [false, true] {
            let terminal = Terminal::new(20, 3, 10).unwrap();
            let calls = Rc::new(Cell::new(0));
            terminal
                .with_parser(|api| {
                    let count = calls.clone();
                    let callback = move || {
                        count.set(count.get() + 1);
                        HandlerResult::Ready(true)
                    };
                    match family {
                        0 => {
                            api.register_csi_handler(FunctionIdentifier::new("Z"), move |_| {
                                callback()
                            })
                            .unwrap();
                        }
                        1 => {
                            api.register_dcs_handler(FunctionIdentifier::new("Z"), move |_, _| {
                                callback()
                            })
                            .unwrap();
                        }
                        2 => {
                            api.register_esc_handler(FunctionIdentifier::new("Z"), callback)
                                .unwrap();
                        }
                        3 => {
                            api.register_osc_handler(666.0, move |_| callback())
                                .unwrap();
                        }
                        4 => {
                            api.register_apc_handler(FunctionIdentifier::new("Z"), move |_| {
                                callback()
                            })
                            .unwrap();
                        }
                        _ => unreachable!(),
                    }
                })
                .unwrap();
            if bytes {
                for byte in sequence.bytes() {
                    terminal.write_bytes(&[byte]).unwrap();
                }
            } else {
                for unit in sequence.encode_utf16() {
                    terminal.write_utf16(&[unit]).unwrap();
                }
            }
            assert_eq!(calls.get(), 1, "family {family}, bytes {bytes}");
        }
    }
}
