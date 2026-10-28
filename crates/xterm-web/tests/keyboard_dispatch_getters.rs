// Licensed under MIT. Source getter ordering must survive fallible DOM adapters.
use std::cell::RefCell;
use xterm_core::input::keyboard::KeyboardEvent;
use xterm_web::input::keyboard_dispatch::*;

struct GetterHost {
    mac: bool,
    windows: bool,
    veto: bool,
    calls: RefCell<Vec<&'static str>>,
}
impl GetterHost {
    fn record(&self, name: &'static str) {
        self.calls.borrow_mut().push(name);
    }
}
impl KeyboardReadHost for GetterHost {
    type Error = &'static str;
    fn snapshot(&self) -> Result<KeyboardSnapshot, Self::Error> {
        Err("eager snapshot forbidden")
    }
    fn is_mac(&self) -> Result<bool, Self::Error> {
        self.record("mac");
        Ok(self.mac)
    }
    fn is_windows(&self) -> Result<bool, Self::Error> {
        self.record("windows");
        Ok(self.windows)
    }
    fn mac_option_is_meta(&self) -> Result<bool, Self::Error> {
        self.record("meta");
        Err("meta getter")
    }
    fn scroll_on_user_input(&self) -> Result<bool, Self::Error> {
        self.record("scroll");
        Ok(false)
    }
    fn alt_graph_state(
        &self,
        _: &dyn KeyEventSource<Error = Self::Error>,
    ) -> Result<bool, Self::Error> {
        self.record("altgraph");
        Err("AltGraph getter")
    }
}
impl KeyboardHost for GetterHost {
    fn custom_key_handler(
        &self,
        _: &dyn KeyEventSource<Error = Self::Error>,
    ) -> Result<bool, Self::Error> {
        self.record("custom");
        Ok(!self.veto)
    }
    fn composition_key_down(
        &self,
        _: &dyn KeyEventSource<Error = Self::Error>,
    ) -> Result<bool, Self::Error> {
        self.record("composition");
        Ok(false)
    }
    fn perform(
        &self,
        _: Option<&dyn KeyEventSource<Error = Self::Error>>,
        _: KeyboardAction<'_>,
    ) -> Result<(), Self::Error> {
        Err("no effect permitted on this rejected route")
    }
}
fn event() -> DispatchKeyEvent<'static> {
    DispatchKeyEvent {
        key: KeyboardEvent {
            key_code: 65,
            key: &[97],
            ..KeyboardEvent::default()
        },
        kind: KeyEventKind::Down,
        repeat: false,
        alt_graph: false,
        char_code: None,
        which: None,
    }
}
#[test]
fn veto_and_composition_short_circuit_unread_throwing_options() {
    for (mac, veto, expected, calls) in [
        (true, true, Ok(Some(false)), vec!["custom"]),
        (
            false,
            false,
            Ok(Some(false)),
            vec!["custom", "mac", "composition", "scroll"],
        ),
        (
            true,
            false,
            Err("meta getter"),
            vec!["custom", "mac", "meta"],
        ),
    ] {
        let dispatch = KeyboardDispatch::default();
        let host = GetterHost {
            mac,
            windows: false,
            veto,
            calls: RefCell::new(Vec::new()),
        };
        assert_eq!(dispatch.key_down(&event(), &host), expected);
        assert_eq!(*host.calls.borrow(), calls);
        assert!(dispatch.state().key_down_seen);
        assert!(!dispatch.state().key_down_handled);
    }
}
#[test]
fn windows_alt_control_short_circuits_altgraph_but_meta_does_not() {
    let host = GetterHost {
        mac: false,
        windows: true,
        veto: false,
        calls: RefCell::new(Vec::new()),
    };
    let mut event = event();
    event.key.alt_key = true;
    event.key.ctrl_key = true;
    assert_eq!(third_level_shift(&host, &event), Ok(true));
    assert_eq!(*host.calls.borrow(), ["mac", "windows"]);
    host.calls.borrow_mut().clear();
    event.key.meta_key = true;
    assert_eq!(third_level_shift(&host, &event), Err("AltGraph getter"));
    assert_eq!(
        *host.calls.borrow(),
        ["mac", "windows", "windows", "altgraph"]
    );
}
