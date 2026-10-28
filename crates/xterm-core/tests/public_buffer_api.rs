use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::{BufferError, BufferSet},
    lifecycle::Disposable,
    options::OptionsService,
    public_api::{BufferNamespaceApi, BufferType},
};

fn core() -> Rc<BufferSet> {
    Rc::new(BufferSet::new(5, 3, Rc::new(OptionsService::default())).unwrap())
}

#[test]
fn views_are_live_and_return_real_lines_and_independent_zero_cells() {
    let core = core();
    let namespace = BufferNamespaceApi::new(core.clone());
    let normal = namespace.normal();
    assert!(Rc::ptr_eq(&normal, &namespace.normal()));
    assert!(Rc::ptr_eq(&normal, &namespace.active().unwrap()));
    assert_eq!(normal.buffer_type(), BufferType::Normal);
    assert_eq!(normal.len().unwrap(), 3);
    let backing = core.normal();
    {
        let mut backing = backing.borrow_mut();
        backing.x = 5;
        backing.y = 2;
        backing.ydisp = 1;
        backing.ybase = 4;
    }
    assert_eq!(normal.cursor_x().unwrap(), 5);
    assert_eq!(normal.cursor_y().unwrap(), 2);
    assert_eq!(normal.viewport_y().unwrap(), 1);
    assert_eq!(normal.base_y().unwrap(), 4);
    assert!(Rc::ptr_eq(
        &normal.get_line(0).unwrap().unwrap(),
        &backing.borrow().lines.get(0).unwrap()
    ));
    assert!(normal.get_line(3).unwrap().is_none());
    let capacity = backing.borrow().lines.max_length();
    assert!(Rc::ptr_eq(
        &normal.get_line(0).unwrap().unwrap(),
        &normal.get_line(capacity).unwrap().unwrap()
    ));
    let mut cell = normal.get_null_cell();
    assert_eq!(cell.content, 0);
    cell.content = 123;
    assert_eq!(normal.get_null_cell().content, 0);
    let stale = normal.get_line(2).unwrap().unwrap();
    backing.borrow().lines.pop().unwrap();
    assert_eq!(normal.len().unwrap(), 2);
    assert!(Rc::ptr_eq(&stale, &normal.get_line(2).unwrap().unwrap()));
    let borrow = backing.borrow_mut();
    assert!(matches!(normal.cursor_x(), Err(BufferError::BorrowedLine)));
    drop(borrow);
}

#[test]
fn namespace_events_share_view_identity_refresh_on_reset_and_stop_on_disposal() {
    let core = core();
    let namespace = Rc::new(BufferNamespaceApi::new(core.clone()));
    let normal = namespace.normal();
    let alternate = namespace.alternate();
    let events = Rc::new(RefCell::new(Vec::new()));
    let captured = events.clone();
    let weak = Rc::downgrade(&namespace);
    let listener = namespace.on_buffer_change().subscribe(move |view| {
        assert!(Rc::ptr_eq(view, &weak.upgrade().unwrap().active().unwrap()));
        captured.borrow_mut().push(view.buffer_type());
    });
    core.activate_alt_buffer(None).unwrap();
    assert!(Rc::ptr_eq(&alternate, &namespace.active().unwrap()));
    core.activate_alt_buffer(None).unwrap();
    core.activate_normal_buffer().unwrap();
    core.activate_alt_buffer(None).unwrap();
    let old_line = normal.get_line(0).unwrap().unwrap();
    core.reset().unwrap();
    assert!(Rc::ptr_eq(&normal, &namespace.active().unwrap()));
    assert!(!Rc::ptr_eq(
        &old_line,
        &normal.get_line(0).unwrap().unwrap()
    ));
    assert_eq!(
        *events.borrow(),
        [
            BufferType::Alternate,
            BufferType::Normal,
            BufferType::Alternate,
            BufferType::Normal
        ]
    );
    listener.dispose();
    core.activate_alt_buffer(None).unwrap();
    assert_eq!(events.borrow().len(), 4);
    let captured = events.clone();
    namespace
        .on_buffer_change()
        .subscribe(move |view| captured.borrow_mut().push(view.buffer_type()));
    namespace.dispose();
    namespace.dispose();
    core.activate_normal_buffer().unwrap();
    assert_eq!(events.borrow().len(), 4);
}
