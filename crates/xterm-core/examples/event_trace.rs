//! Event operation trace runner for scripts/check-events-and-circular-list.mjs.
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::io::{self, BufRead};
use std::rc::Rc;
use xterm_core::event::Emitter;
use xterm_core::lifecycle::{Disposable, DisposableHandle};

type Handles = Rc<RefCell<HashMap<u32, DisposableHandle>>>;
type Log = Rc<RefCell<Vec<String>>>;

#[derive(Clone, Copy)]
enum Action {
    Normal,
    Remove(u32),
    Add(u32),
    Nested,
    Dispose,
}

fn subscribe(emitter: &Rc<Emitter<i32>>, handles: &Handles, log: &Log, id: u32, action: Action) {
    let weak_emitter = Rc::downgrade(emitter);
    let weak_handles = Rc::downgrade(handles);
    let output = Rc::clone(log);
    let handle = emitter.event().try_subscribe(move |value| {
        output.borrow_mut().push(format!("{id}:{value}"));
        if let (Some(emitter), Some(handles)) = (weak_emitter.upgrade(), weak_handles.upgrade()) {
            match action {
                Action::Normal => {}
                Action::Remove(target) => {
                    let handle = handles.borrow().get(&target).cloned();
                    if let Some(handle) = handle {
                        handle.dispose();
                    }
                }
                Action::Add(target) => {
                    subscribe(&emitter, &handles, &output, target, Action::Normal)
                }
                Action::Nested if *value == 1 => emitter.fire(&2)?,
                Action::Nested => {}
                Action::Dispose => emitter.dispose(),
            }
        }
        Ok(())
    });
    handles.borrow_mut().insert(id, handle);
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut emitter = Rc::new(Emitter::new());
    let handles: Handles = Rc::default();
    let log: Log = Rc::default();
    for line in io::stdin().lock().lines() {
        let line = line?;
        let fields: Vec<_> = line.split('\t').collect();
        let mut result = "ok";
        match fields[0] {
            "N" => {
                emitter = Rc::new(Emitter::new());
                handles.borrow_mut().clear();
            }
            "A" => {
                let id = fields[1].parse()?;
                let action = match fields[2] {
                    "normal" => Action::Normal,
                    "remove" => Action::Remove(fields[3].parse()?),
                    "add" => Action::Add(fields[3].parse()?),
                    "nested" => Action::Nested,
                    "dispose" => Action::Dispose,
                    action => return Err(format!("unknown action {action}").into()),
                };
                subscribe(&emitter, &handles, &log, id, action);
            }
            "K" => {
                let id: u32 = fields[1].parse()?;
                let handle = handles.borrow().get(&id).cloned();
                if let Some(handle) = handle {
                    handle.dispose();
                }
            }
            "F" => {
                if emitter.fire(&fields[1].parse()?).is_err() {
                    result = "error";
                }
            }
            "X" => emitter.dispose(),
            command => return Err(format!("unknown command {command}").into()),
        }
        println!("{result}|{}", log.borrow().join(","));
        log.borrow_mut().clear();
    }
    Ok(())
}
