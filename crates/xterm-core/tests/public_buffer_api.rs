use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
use xterm_core::{
    buffer::{constants::bg_flags, Buffer, BufferError, BufferSet},
    lifecycle::Disposable,
    options::OptionsService,
    public_api::{BufferApiView, BufferNamespaceApi, BufferType},
};

fn core() -> Rc<BufferSet> {
    Rc::new(BufferSet::new(5, 3, Rc::new(OptionsService::default())).unwrap())
}

struct ReentrantPayload {
    view: Weak<BufferApiView>,
    observed: Rc<Cell<Option<usize>>>,
}
impl Drop for ReentrantPayload {
    fn drop(&mut self) {
        self.observed
            .set(Some(self.view.upgrade().unwrap().cursor_x().unwrap()));
    }
}

#[test]
fn init_releases_old_payload_after_publishing_new_backing_without_a_borrow() {
    let options = Rc::new(OptionsService::default());
    let mut old = Buffer::new(false, 5, 1, options.clone()).unwrap();
    old.fill_viewport_rows(None).unwrap();
    let old = Rc::new(RefCell::new(old));
    let view = Rc::new(BufferApiView::new(old.clone(), BufferType::Normal));
    let observed = Rc::new(Cell::new(None));
    {
        let mut cell = old.borrow().get_null_cell(None);
        cell.bg |= bg_flags::HAS_EXTENDED;
        cell.extended.set_payload(Some(Rc::new(ReentrantPayload {
            view: Rc::downgrade(&view),
            observed: observed.clone(),
        })));
        old.borrow()
            .lines
            .get(0)
            .unwrap()
            .borrow_mut()
            .set_cell(0, &cell);
    }
    drop(old);
    assert_eq!(observed.get(), None);
    let mut replacement = Buffer::new(false, 5, 1, options).unwrap();
    replacement.x = 2;
    view.init(Rc::new(RefCell::new(replacement)));
    assert_eq!(observed.get(), Some(2));
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
