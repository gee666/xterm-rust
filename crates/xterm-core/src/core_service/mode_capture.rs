//! A read-only JS object capture without holding a RefCell borrow across an
//! observable getter. In-place writes remain visible; reset replaces the source
//! object, so surviving captures freeze the old value at that boundary.
use super::{CoreService, DecPrivateModes};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

type FrozenModes = RefCell<Option<DecPrivateModes>>;
pub(super) type ModeCaptures = RefCell<Vec<Weak<FrozenModes>>>;

pub struct PrivateModesCapture<'a> {
    current: &'a RefCell<DecPrivateModes>,
    frozen: Rc<FrozenModes>,
}
impl PrivateModesCapture<'_> {
    pub fn read(&self) -> DecPrivateModes {
        match self.frozen.borrow().as_ref() {
            Some(value) => value.clone(),
            None => self.current.borrow().clone(),
        }
    }
}
impl CoreService {
    /// Use across an observable option getter, not a long-lived RefCell borrow.
    /// The lifetime keeps the core allocation alive; it does not block reset.
    pub fn capture_private_modes(&self) -> PrivateModesCapture<'_> {
        let frozen = Rc::new(RefCell::new(None));
        let mut captures = self.private_modes_captures.borrow_mut();
        captures.retain(|capture| capture.strong_count() > 0);
        captures.push(Rc::downgrade(&frozen));
        PrivateModesCapture {
            current: &self.dec_private_modes,
            frozen,
        }
    }

    pub(super) fn freeze_private_modes_captures(&self) {
        for capture in self.private_modes_captures.take() {
            if let Some(capture) = capture.upgrade() {
                // Each capture is removed from the live list at its first reset.
                // Later resets cannot overwrite its captured source object.
                let previous = self.dec_private_modes.borrow().clone();
                *capture.borrow_mut() = Some(previous);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core_service::{BufferPosition, CursorStyle};
    use crate::log_service::{LogError, LogRecord, LogService, LogSink};
    use crate::options::OptionsService;
    struct NoLogging;
    impl LogSink for NoLogging {
        fn log(&self, _: LogRecord) -> Result<(), LogError> {
            panic!("mode capture must not log")
        }
    }
    fn core() -> CoreService {
        let options = Rc::new(OptionsService::default());
        let logger = Rc::new(LogService::new(Rc::clone(&options), Rc::new(NoLogging)).unwrap());
        CoreService::new(options, logger, BufferPosition::default).unwrap()
    }

    #[test]
    fn in_place_changes_stay_visible_until_first_reset() {
        let core = core();
        let captured = core.capture_private_modes();
        assert_eq!(captured.read(), DecPrivateModes::default());
        {
            let mut modes = core.dec_private_modes.borrow_mut();
            modes.win32_input_mode = true;
            modes.application_cursor_keys = true;
            modes.cursor_blink = Some(false);
            modes.cursor_style = Some(CursorStyle::Bar);
            modes.wraparound = false;
        }
        let previous = core.dec_private_modes.borrow().clone();
        assert_eq!(captured.read(), previous);
        core.reset();
        assert_eq!(captured.read(), previous);
        assert_eq!(*core.dec_private_modes.borrow(), DecPrivateModes::default());
        core.dec_private_modes.borrow_mut().origin = true;
        assert_eq!(captured.read(), previous);
    }

    #[test]
    fn nested_captures_keep_their_own_replacement_boundaries() {
        let core = core();
        core.dec_private_modes.borrow_mut().win32_input_mode = true;
        let first = core.capture_private_modes();
        core.reset();
        let second = core.capture_private_modes();
        core.dec_private_modes.borrow_mut().origin = true;
        assert!(second.read().origin);
        core.reset();
        assert!(first.read().win32_input_mode);
        assert!(!first.read().origin);
        assert!(second.read().origin);
        assert!(!second.read().win32_input_mode);
        let third = core.capture_private_modes();
        core.reset();
        core.reset();
        assert_eq!(third.read(), DecPrivateModes::default());
        assert!(first.read().win32_input_mode);
        assert!(second.read().origin);
    }

    #[test]
    fn dropped_captures_do_not_accumulate_or_hold_a_mode_borrow() {
        let core = core();
        for _ in 0..1000 {
            let captured = core.capture_private_modes();
            assert_eq!(core.private_modes_captures.borrow().len(), 1);
            core.dec_private_modes.borrow_mut().wraparound = false;
            assert!(!captured.read().wraparound);
        }
        core.reset();
        assert!(core.private_modes_captures.borrow().is_empty());
        assert_eq!(*core.dec_private_modes.borrow(), DecPrivateModes::default());
    }
}
