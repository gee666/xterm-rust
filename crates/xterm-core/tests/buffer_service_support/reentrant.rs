// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
#[path = "mod.rs"]
mod buffers;
use buffers::buffer_snapshot;
use std::rc::Rc;
use xterm_core::buffer::{Buffer, BufferService};

pub fn service_snapshot(service: &BufferService, tag: &str) -> String {
    let set = &service.buffers;
    let active = service.buffer();
    let kind = if Rc::ptr_eq(&active, &set.normal()) {
        "n"
    } else if Rc::ptr_eq(&active, &set.alt()) {
        "a"
    } else {
        "old"
    };
    format!(
        "{tag}#{}#{}#{}#{kind}#{}#{}#{}#{}#{}#{}",
        service.cols(),
        service.rows(),
        u8::from(service.is_user_scrolling.get()),
        buffer_snapshot(&set.normal().borrow()),
        buffer_snapshot(&set.alt().borrow()),
        buffer_snapshot(&active.borrow()),
        markers(&set.normal().borrow()),
        markers(&set.alt().borrow()),
        markers(&active.borrow())
    )
}
fn markers(buffer: &Buffer) -> String {
    let markers = buffer.markers.borrow();
    if markers.is_empty() {
        "-".into()
    } else {
        markers
            .iter()
            .map(|marker| format!("{},{}", marker.line(), u8::from(marker.is_disposed())))
            .collect::<Vec<_>>()
            .join(";")
    }
}

use std::cell::RefCell;
use xterm_core::options::{OptionValue, OptionsService};
pub fn new_service() -> Rc<BufferService> {
    Rc::new(
        BufferService::new(Rc::new(OptionsService::new(
            false,
            [
                ("cols", OptionValue::Number(4.0)),
                ("rows", OptionValue::Number(3.0)),
                ("scrollback", OptionValue::Number(1.0)),
            ],
        )))
        .unwrap(),
    )
}
pub type Save = Rc<dyn Fn(&str)>;
pub fn recorder(service: &Rc<BufferService>, out: &Rc<RefCell<Vec<String>>>) -> Save {
    let service = Rc::downgrade(service);
    let out = Rc::clone(out);
    Rc::new(move |tag| {
        out.borrow_mut()
            .push(service_snapshot(&service.upgrade().unwrap(), tag))
    })
}
pub fn on_scroll(service: &BufferService, save: Save) {
    service.on_scroll().subscribe(move |_| save("scroll"));
}
