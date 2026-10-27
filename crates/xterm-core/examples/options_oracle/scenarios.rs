// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use xterm_core::lifecycle::Disposable;
use xterm_core::options::{OptionError, OptionValue as V, OptionsService};

pub fn run() -> Result<String, OptionError> {
    let service = Rc::new(OptionsService::default());
    let log = Rc::new(RefCell::new(Vec::new()));
    let weak = Rc::downgrade(&service);
    let output = log.clone();
    service.on_option_change().subscribe(move |key| {
        output.borrow_mut().push(format!("general:{key}"));
        let service = weak
            .upgrade()
            .expect("service exists during its own dispatch");
        if *key == "cols" && service.get("cols").unwrap() == V::Number(100.0) {
            service.set("cols", V::Number(101.0)).unwrap();
        }
    });
    let output = log.clone();
    service.on_specific_option_change("cols", move |value| {
        if let V::Number(n) = value {
            output.borrow_mut().push(format!("specific:{n}"));
        }
    })?;
    let output = log.clone();
    service.on_multiple_option_change(&["cols", "cols", "rows"], move || {
        output.borrow_mut().push("multiple".into())
    })?;
    let temporary = service
        .on_option_change()
        .subscribe(|_| panic!("unsubscribed listener"));
    temporary.dispose();
    service.set("cols", V::Number(100.0))?;
    service.set_raw("rows", V::Number(-1.0))?;
    service.set("rows", V::Number(24.0))?;
    let first = service.get("windowsPty")?;
    service.set("windowsPty", V::Null)?;
    let second = service.get("windowsPty")?;
    service.set("windowsPty", V::Undefined)?;
    let third = service.get("windowsPty")?;
    log.borrow_mut()
        .push(format!("fresh:{}:{}", first != second, second != third));
    service.set("documentOverride", V::object(()))?;
    service.set("linkHandler", V::object(()))?;
    service.dispose();
    service.dispose();
    log.borrow_mut().push(format!(
        "clean:{}:{}",
        service.get("documentOverride")? == V::Null,
        service.get("linkHandler")? == V::Null
    ));
    service
        .on_option_change()
        .subscribe(|_| panic!("late disposed listener"));
    service.set("rows", V::Number(30.0))?;
    log.borrow_mut()
        .push(format!("after:{}", service.get("rows")? == V::Number(30.0)));

    // Disposing during a two-listener dispatch clears the active array. The
    // shared emitter reports the same failure where JS throws a TypeError.
    let disposing = Rc::new(OptionsService::default());
    let weak = Rc::downgrade(&disposing);
    disposing
        .on_option_change()
        .subscribe(move |_| weak.upgrade().unwrap().dispose());
    let called = Rc::new(Cell::new(false));
    let output = called.clone();
    disposing
        .on_option_change()
        .subscribe(move |_| output.set(true));
    let failed = matches!(
        disposing.set("rows", V::Number(40.0)),
        Err(OptionError::Dispatch(_))
    );
    log.borrow_mut()
        .push(format!("during:{failed}:{}", called.get()));
    let trace = log.borrow().join(",");
    Ok(trace)
}
