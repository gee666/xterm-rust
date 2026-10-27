use std::cell::{Cell, RefCell};
use std::rc::Rc;
use xterm_core::core_service::{BufferPosition, CoreService};
use xterm_core::log_service::{LogError, LogRecord, LogService, LogSink};
use xterm_core::options::{OptionValue, OptionsService};

#[derive(Default)]
pub struct Records {
    pub logs: RefCell<Vec<LogRecord>>,
    pub order: RefCell<Vec<String>>,
}
impl LogSink for Records {
    fn log(&self, record: LogRecord) -> Result<(), LogError> {
        self.order.borrow_mut().push(format!("{:?}", record.level));
        self.logs.borrow_mut().push(record);
        Ok(())
    }
}

pub struct Harness {
    pub options: Rc<OptionsService>,
    pub logger: Rc<LogService>,
    pub core: Rc<CoreService>,
    pub records: Rc<Records>,
    pub position: Rc<Cell<BufferPosition>>,
}
impl Harness {
    pub fn new(options: impl IntoIterator<Item = (&'static str, OptionValue)>) -> Self {
        let options = Rc::new(OptionsService::new(false, options));
        let records = Rc::new(Records::default());
        let logger = Rc::new(LogService::new(Rc::clone(&options), records.clone()).unwrap());
        let position = Rc::new(Cell::new(BufferPosition::default()));
        let read_position = Rc::clone(&position);
        let core = Rc::new(
            CoreService::new(Rc::clone(&options), Rc::clone(&logger), move || {
                read_position.get()
            })
            .unwrap(),
        );
        Self {
            options,
            logger,
            core,
            records,
            position,
        }
    }
    pub fn listen(&self) {
        let records = Rc::clone(&self.records);
        self.core
            .on_request_scroll_to_bottom()
            .subscribe(move |_| records.order.borrow_mut().push("scroll".into()));
        let records = Rc::clone(&self.records);
        self.core
            .on_user_input()
            .subscribe(move |_| records.order.borrow_mut().push("user".into()));
        let records = Rc::clone(&self.records);
        self.core
            .on_data()
            .subscribe(move |_| records.order.borrow_mut().push("data".into()));
        let records = Rc::clone(&self.records);
        self.core
            .on_binary()
            .subscribe(move |_| records.order.borrow_mut().push("binary".into()));
    }
}
