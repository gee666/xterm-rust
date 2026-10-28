use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::{AttributeData, BufferLine},
    circular_list::CircularList,
    platform::{get_zoom_factor, Navigator, Platform},
    version::XTERM_VERSION,
    windows_mode::update_windows_mode_wrapped_state,
};

#[test]
fn node_navigator_precedence_and_version() {
    let node = Platform::detect(true, None).unwrap();
    assert!(node.is_node);
    assert!(!node.is_windows);
    let fake_node = Navigator {
        user_agent: "Node.js/24",
        platform: "Win32",
    };
    assert!(Platform::detect(true, Some(fake_node)).unwrap().is_node);
    assert!(Platform::detect(false, Some(fake_node)).unwrap().is_windows);
    assert!(Platform::detect(false, None).is_err());
    let browser = Navigator {
        user_agent: "Firefox",
        platform: "Linux x86_64",
    };
    let platform = Platform::detect(true, Some(browser)).unwrap();
    assert!(!platform.is_node);
    assert!(platform.is_firefox);
    assert!(platform.is_linux);
    assert_eq!(XTERM_VERSION, "6.0.0");
    for ratio in [None, Some(0.0), Some(2.0), Some(f64::NAN)] {
        assert_eq!(get_zoom_factor(ratio), 1.0);
    }
}
#[test]
fn safari_regex_boundaries_and_ascii_word_boundary() {
    for (ua, safari, version, chrome_os) in [
        ("Safari Version/17.5", true, 17.0, false),
        ("Safari Chrome Version/x Version/18", true, 18.0, false),
        ("Chrome Safari Version/19", false, 0.0, false),
        ("android SAFARI", false, 0.0, false),
        ("\nSafari", false, 0.0, false),
        ("CrOS Safari Version/4", true, 4.0, true),
        ("éCrOSé Safari", true, 0.0, true),
        ("_CrOS Safari", true, 0.0, false),
    ] {
        let p = Platform::detect(
            false,
            Some(Navigator {
                user_agent: ua,
                platform: "MacIntel",
            }),
        )
        .unwrap();
        assert_eq!(p.is_safari, safari, "{ua}");
        assert_eq!(p.safari_version, version, "{ua}");
        assert_eq!(p.is_chrome_os, chrome_os, "{ua}");
        assert!(p.is_mac);
    }
}
fn lines() -> CircularList<Rc<RefCell<BufferLine>>> {
    let lines = CircularList::new(5).unwrap();
    for _ in 0..5 {
        lines
            .push(Rc::new(RefCell::new(BufferLine::new(10, None, false))))
            .unwrap();
    }
    lines
}
#[test]
fn upstream_previous_non_whitespace_marks_wrapped() {
    let lines = lines();
    let line = lines.get(0).unwrap();
    for i in 0..10 {
        line.borrow_mut()
            .set_cell_from_codepoint(i, 97, 1, &AttributeData::default());
    }
    update_windows_mode_wrapped_state(&lines, 0, 1, 10);
    assert!(lines.get(1).unwrap().borrow().is_wrapped);
}
#[test]
fn upstream_previous_space_does_not_mark_wrapped() {
    let lines = lines();
    let line = lines.get(0).unwrap();
    for i in 0..9 {
        line.borrow_mut()
            .set_cell_from_codepoint(i, 97, 1, &AttributeData::default());
    }
    line.borrow_mut()
        .set_cell_from_codepoint(9, 32, 1, &AttributeData::default());
    update_windows_mode_wrapped_state(&lines, 0, 1, 10);
    assert!(!lines.get(1).unwrap().borrow().is_wrapped);
}
#[test]
fn upstream_previous_null_does_not_mark_wrapped() {
    let lines = lines();
    update_windows_mode_wrapped_state(&lines, 0, 1, 10);
    assert!(!lines.get(1).unwrap().borrow().is_wrapped);
}
#[test]
fn wrap_boundaries_and_combined_codepoints() {
    let lines = lines();
    lines.get(1).unwrap().borrow_mut().is_wrapped = true;
    update_windows_mode_wrapped_state(&lines, 0, 1, 0);
    update_windows_mode_wrapped_state(&lines, 0, 0, 10);
    assert!(!lines.get(1).unwrap().borrow().is_wrapped);
    lines.get(1).unwrap().borrow_mut().is_wrapped = true;
    update_windows_mode_wrapped_state(&lines, 0, 1, 11);
    assert!(!lines.get(1).unwrap().borrow().is_wrapped);
    for (code, expected) in [
        (0, false),
        (32, false),
        (9, true),
        (0x2000, true),
        (0x1f600, true),
    ] {
        lines.get(2).unwrap().borrow_mut().set_cell_from_codepoint(
            9,
            code,
            1,
            &AttributeData::default(),
        );
        update_windows_mode_wrapped_state(&lines, 2, 1, 10);
        assert_eq!(lines.get(3).unwrap().borrow().is_wrapped, expected);
    }
    let line = lines.get(2).unwrap();
    line.borrow_mut().add_codepoint_to_cell(9, 32, 0);
    update_windows_mode_wrapped_state(&lines, 2, 1, 10);
    assert!(!lines.get(3).unwrap().borrow().is_wrapped);
}
#[test]
fn signed_previous_ring_slot_and_empty_combined_nan() {
    use xterm_core::buffer::{constants::content, CellData};
    let rows = CircularList::new(2).unwrap();
    for code in [0, 0, 97] {
        let mut line = BufferLine::new(10, None, false);
        line.set_cell_from_codepoint(9, code, 1, &AttributeData::default());
        rows.push(Rc::new(RefCell::new(line))).unwrap();
    }
    update_windows_mode_wrapped_state(&rows, 0, 0, 10);
    assert!(rows.get(0).unwrap().borrow().is_wrapped);
    let cell = CellData {
        content: content::IS_COMBINED_MASK | (1 << content::WIDTH_SHIFT),
        ..CellData::default()
    };
    rows.get(0).unwrap().borrow_mut().set_cell(9, &cell);
    update_windows_mode_wrapped_state(&rows, 0, 1, 10);
    assert!(rows.get(1).unwrap().borrow().is_wrapped);
}
