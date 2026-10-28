use std::rc::Rc;
use xterm_core::{
    headless::Terminal,
    options::OptionValue,
    serialize::{AddonSerializeError, SerializeAddon, SerializeOptions},
};

#[test]
fn addon_back_reference_in_options_does_not_keep_native_state_in_a_cycle() {
    let terminal = Terminal::new(10, 2, 20).unwrap();
    terminal
        .options()
        .set("allowProposedApi", OptionValue::Bool(true))
        .unwrap();
    let addon = Rc::new(SerializeAddon::new());
    let weak_addon = Rc::downgrade(&addon);
    terminal.load_addon(addon.clone()).unwrap();
    // A native options payload may retain an addon exactly as a JS option
    // object can. The addon must not strongly retain the state containing it.
    terminal
        .options()
        .set("theme", OptionValue::object(addon.clone()))
        .unwrap();
    let weak_state = Rc::downgrade(&terminal.state_handle_for_addon());
    terminal.write("live").unwrap();
    assert!(addon.serialize(&SerializeOptions::default()).is_ok());
    drop(terminal);
    assert!(weak_state.upgrade().is_none());
    assert!(matches!(
        addon.serialize(&SerializeOptions::default()),
        Err(AddonSerializeError::TerminalReleased)
    ));
    drop(addon);
    assert!(weak_addon.upgrade().is_none());
}
