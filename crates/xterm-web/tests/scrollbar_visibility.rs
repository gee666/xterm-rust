use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;
use xterm_core::options::OptionValue;
use xterm_web::links::time_debounce::TimerScheduler;
use xterm_web::scroll::visibility::{VisibilityController, VisibilityError};
use xterm_web::scroll::ScrollbarVisibility;

#[derive(Default)]
struct Host {
    next: usize,
    pending: BTreeSet<usize>,
    events: Vec<String>,
    fail_request: bool,
    fail_cancel: bool,
}
#[derive(Clone)]
struct Clock(Rc<RefCell<Host>>);
impl TimerScheduler for Clock {
    type Handle = usize;
    type Error = &'static str;
    fn now(&self) -> f64 {
        0.0
    }
    fn request_timeout(&mut self, delay: f64) -> Result<usize, Self::Error> {
        let mut host = self.0.borrow_mut();
        host.events.push(format!("request:{delay}"));
        if host.fail_request {
            return Err("request failed");
        }
        host.next += 1;
        let token = host.next;
        host.pending.insert(token);
        Ok(token)
    }
    fn cancel_timeout(&mut self, token: usize) -> Result<(), Self::Error> {
        let mut host = self.0.borrow_mut();
        host.events.push(format!("cancel:{token}"));
        if host.fail_cancel {
            return Err("cancel failed");
        }
        host.pending.remove(&token);
        Ok(())
    }
    fn cleanup_error(&mut self, error: Self::Error) {
        self.0.borrow_mut().events.push(format!("cleanup:{error}"));
    }
}
type Controller = VisibilityController<Clock>;
fn controller(
    mode: ScrollbarVisibility,
) -> (Controller, Rc<RefCell<Host>>, Rc<RefCell<Vec<String>>>) {
    let host = Rc::new(RefCell::new(Host::default()));
    let node = Rc::new(RefCell::new(Vec::new()));
    let value = Controller::new(
        Clock(Rc::clone(&host)),
        mode,
        "visible".into(),
        "hidden".into(),
    );
    let writes = Rc::clone(&node);
    value
        .set_dom_node(move |name| {
            writes.borrow_mut().push(name.to_owned());
            Ok(())
        })
        .unwrap();
    (value, host, node)
}
fn fire(controller: &Controller, host: &RefCell<Host>) {
    let token = *host.borrow().pending.first().unwrap();
    host.borrow_mut().pending.remove(&token);
    controller.on_timeout().unwrap();
}

#[test]
fn reveal_is_deferred_and_hide_uses_distinct_fade_rules() {
    let (value, host, writes) = controller(ScrollbarVisibility::Auto);
    assert_eq!(*writes.borrow(), ["hidden"]);
    value.set_should_be_visible(true).unwrap();
    assert!(!value.is_visible());
    value.set_is_needed(true).unwrap();
    assert!(value.is_visible());
    assert!(value.has_pending_reveal());
    assert_eq!(*writes.borrow(), ["hidden"]);
    assert_eq!(host.borrow().events, ["request:0"]);
    fire(&value, &host);
    assert_eq!(*writes.borrow(), ["hidden", "visible"]);
    value.set_should_be_visible(false).unwrap();
    assert_eq!(writes.borrow().last().unwrap(), "hidden xterm-fade");
    value.set_should_be_visible(true).unwrap();
    value.set_is_needed(false).unwrap();
    assert_eq!(writes.borrow().last().unwrap(), "hidden");
    assert!(host.borrow().pending.is_empty());
}

#[test]
fn pending_reveal_uses_current_target_but_target_replacement_does_not_force_reveal() {
    let (value, host, old) = controller(ScrollbarVisibility::Visible);
    value.set_is_needed(true).unwrap();
    let next = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&next);
    value
        .set_dom_node(move |name| {
            output.borrow_mut().push(name.to_owned());
            Ok(())
        })
        .unwrap();
    fire(&value, &host);
    assert_eq!(*old.borrow(), ["hidden"]);
    assert_eq!(*next.borrow(), ["hidden", "visible"]);
    let replacement = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&replacement);
    value
        .set_dom_node(move |name| {
            output.borrow_mut().push(name.to_owned());
            Ok(())
        })
        .unwrap();
    value.ensure_visibility().unwrap();
    assert_eq!(*replacement.borrow(), ["hidden"]);
    assert!(value.is_visible());
}

#[test]
fn visibility_strings_do_not_coerce_to_known_numeric_modes() {
    let (value, host, _) = controller(ScrollbarVisibility::from_value(OptionValue::from("2")));
    value.set_is_needed(true).unwrap();
    value.set_should_be_visible(true).unwrap();
    assert!(value.is_visible());
    fire(&value, &host);
    value.set_visibility(ScrollbarVisibility::Hidden).unwrap();
    assert!(!value.is_visible());
    value
        .set_visibility(ScrollbarVisibility::from_value(OptionValue::Null))
        .unwrap();
    assert!(value.is_visible());
}

#[test]
fn request_failure_retains_source_committed_visible_flag() {
    let (value, host, _) = controller(ScrollbarVisibility::Auto);
    value.set_is_needed(true).unwrap();
    host.borrow_mut().fail_request = true;
    assert_eq!(
        value.set_should_be_visible(true),
        Err(VisibilityError::Host("request failed"))
    );
    assert!(value.is_visible());
    assert!(!value.has_pending_reveal());
    host.borrow_mut().fail_request = false;
    value.ensure_visibility().unwrap();
    assert_eq!(host.borrow().events.len(), 1);
    value.set_is_needed(false).unwrap();
    value.set_is_needed(true).unwrap();
    assert_eq!(host.borrow().events.len(), 2);
}

#[test]
fn cancellation_failure_keeps_pending_and_visible_until_recovery() {
    let (value, host, _) = controller(ScrollbarVisibility::Auto);
    value.set_is_needed(true).unwrap();
    value.set_should_be_visible(true).unwrap();
    host.borrow_mut().fail_cancel = true;
    assert_eq!(
        value.set_is_needed(false),
        Err(VisibilityError::Host("cancel failed"))
    );
    assert!(!value.is_needed());
    assert!(value.is_visible());
    assert!(value.has_pending_reveal());
    host.borrow_mut().fail_cancel = false;
    value.ensure_visibility().unwrap();
    assert!(!value.is_visible());
    assert!(!value.has_pending_reveal());
}

#[test]
fn disposed_timer_errors_after_visible_commit_and_allows_source_hide_path() {
    let (value, _, _) = controller(ScrollbarVisibility::Auto);
    value.set_is_needed(true).unwrap();
    value.dispose().unwrap();
    assert_eq!(
        value.set_should_be_visible(true),
        Err(VisibilityError::DisposedTimer)
    );
    assert!(value.is_visible());
    value.ensure_visibility().unwrap();
    value.set_visibility(ScrollbarVisibility::Hidden).unwrap();
    assert!(!value.is_visible());
    value.dispose().unwrap();
    assert!(value.timer_is_disposed());
}

#[test]
fn class_setters_reenter_without_borrow_panics_and_weak_dispatch_does_not_leak() {
    let host = Rc::new(RefCell::new(Host::default()));
    let value = Rc::new(Controller::new(
        Clock(Rc::clone(&host)),
        ScrollbarVisibility::Auto,
        "visible".into(),
        "hidden".into(),
    ));
    let weak = Rc::downgrade(&value);
    value
        .set_dom_node(move |name| {
            if name == "visible" {
                weak.upgrade().unwrap().set_is_needed(false).unwrap();
            }
            Ok(())
        })
        .unwrap();
    value.set_is_needed(true).unwrap();
    value.set_should_be_visible(true).unwrap();
    fire(&value, &host);
    assert!(!value.is_visible());
    let dispatch = value.weak_dispatcher();
    value.set_is_needed(true).unwrap();
    drop(value);
    assert!(host.borrow().pending.is_empty());
    dispatch().unwrap();
}
