use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    core_service::{BufferPosition, CoreService},
    headless::{ModesSnapshot, MouseTrackingMode},
    log_service::{LogError, LogRecord, LogService, LogSink},
    mouse_state::MouseStateService,
    options::OptionsService,
};

#[derive(Default)]
struct Records(RefCell<Vec<LogRecord>>);
impl LogSink for Records {
    fn log(&self, record: LogRecord) -> Result<(), LogError> {
        self.0.borrow_mut().push(record);
        Ok(())
    }
}
fn core() -> CoreService {
    let options = Rc::new(OptionsService::default());
    let log = Rc::new(LogService::new(Rc::clone(&options), Rc::new(Records::default())).unwrap());
    CoreService::new(options, log, BufferPosition::default).unwrap()
}

#[test]
fn pa05_all_twelve_defaults_and_snapshot_independence() {
    let core = core();
    let mouse = MouseStateService::<()>::new();
    let original = ModesSnapshot::from_services(&core, &mouse).unwrap();
    assert_eq!(
        original,
        ModesSnapshot {
            application_cursor_keys_mode: false,
            application_keypad_mode: false,
            bracketed_paste_mode: false,
            insert_mode: false,
            mouse_tracking_mode: MouseTrackingMode::None,
            origin_mode: false,
            reverse_wraparound_mode: false,
            send_focus_mode: false,
            show_cursor: true,
            synchronized_output_mode: false,
            win32_input_mode: false,
            wraparound_mode: true,
        }
    );
    core.modes.borrow_mut().insert_mode = true;
    {
        let mut modes = core.dec_private_modes.borrow_mut();
        modes.application_cursor_keys = true;
        modes.application_keypad = true;
        modes.bracketed_paste_mode = true;
        modes.origin = true;
        modes.reverse_wraparound = true;
        modes.send_focus = true;
        modes.synchronized_output = true;
        modes.win32_input_mode = true;
        modes.wraparound = false;
    }
    core.is_cursor_hidden.set(true);
    let changed = ModesSnapshot::from_services(&core, &mouse).unwrap();
    assert!(changed.application_cursor_keys_mode && changed.application_keypad_mode);
    assert!(changed.bracketed_paste_mode && changed.insert_mode && changed.origin_mode);
    assert!(changed.reverse_wraparound_mode && changed.send_focus_mode);
    assert!(changed.synchronized_output_mode && changed.win32_input_mode);
    assert!(!changed.show_cursor && !changed.wraparound_mode);
    assert!(original.show_cursor && original.wraparound_mode && !original.insert_mode);
    core.reset();
    // CoreService intentionally retains cursor-hidden state across reset.
    assert!(
        !ModesSnapshot::from_services(&core, &mouse)
            .unwrap()
            .show_cursor
    );
}

#[test]
fn pa05_mouse_protocol_names_project_to_public_lowercase_snapshot() {
    let core = core();
    let mouse = MouseStateService::<()>::new();
    for (protocol, projected) in [
        ("NONE", "none"),
        ("X10", "x10"),
        ("VT200", "vt200"),
        ("DRAG", "drag"),
        ("ANY", "any"),
    ] {
        mouse.set_active_protocol(protocol).unwrap();
        assert_eq!(
            ModesSnapshot::from_services(&core, &mouse)
                .unwrap()
                .mouse_tracking_mode
                .as_str(),
            projected
        );
    }
}

#[test]
fn pa05_conflicting_mode_borrow_is_a_recoverable_error() {
    let core = core();
    let mouse = MouseStateService::<()>::new();
    let _held = core.dec_private_modes.borrow_mut();
    assert!(ModesSnapshot::from_services(&core, &mouse).is_err());
}
