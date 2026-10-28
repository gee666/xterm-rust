//! Reviewer regression: native cleanup must not retain an addon-state borrow.
use std::{
    cell::Cell,
    panic::{catch_unwind, AssertUnwindSafe},
    rc::{Rc, Weak},
};
use xterm_core::{
    unicode_addons::UnicodeGraphemesAddon, unicode_service::UnicodeService, unicode_v6::UnicodeV6,
};

struct ListenerCapture {
    addon: Weak<UnicodeGraphemesAddon>,
    dropped: Rc<Cell<usize>>,
}
impl ListenerCapture {
    fn observe(&self) {
        assert!(self.addon.upgrade().is_some());
    }
}
impl Drop for ListenerCapture {
    fn drop(&mut self) {
        self.addon.upgrade().unwrap().dispose().unwrap();
        self.dropped.set(self.dropped.get() + 1);
    }
}
fn service() -> Rc<UnicodeService> {
    let service = Rc::new(UnicodeService::new());
    service.register(Rc::new(UnicodeV6::new())).unwrap();
    service
}
#[test]
fn replacing_only_owned_service_releases_borrow_before_listener_capture_drop() {
    let addon = Rc::new(UnicodeGraphemesAddon::new());
    let old = service();
    let old_weak = Rc::downgrade(&old);
    let dropped = Rc::new(Cell::new(0));
    let capture = ListenerCapture {
        addon: Rc::downgrade(&addon),
        dropped: Rc::clone(&dropped),
    };
    // The emitter retains this capture after the subscription handle is dropped.
    old.on_change().subscribe(move |_| capture.observe());
    addon.activate(old).unwrap();
    assert_eq!(dropped.get(), 0);
    assert!(old_weak.upgrade().is_some());
    let next = service();
    let result = catch_unwind(AssertUnwindSafe(|| addon.activate(Rc::clone(&next))));
    assert!(
        result.is_ok(),
        "old service cleanup reentered an outstanding addon borrow"
    );
    result.unwrap().unwrap();
    assert!(old_weak.upgrade().is_none());
    assert_eq!(dropped.get(), 1);
    assert_eq!(next.active_version(), "15-graphemes");
    addon.dispose().unwrap();
    assert_eq!(next.active_version(), "6");
}
