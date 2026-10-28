// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Expected traces come from the unchanged source terminal, not a model.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::{
    event::EmitError,
    headless::Terminal,
    options::{OptionObject, OptionValue, OptionsService},
    parser::{escape::FunctionIdentifier, handler::HandlerResult},
};
type Trace = Rc<RefCell<Vec<String>>>;
fn pty() -> OptionValue {
    OptionValue::property_map([
        ("backend", OptionValue::from("conpty")),
        ("buildNumber", OptionValue::Number(21375.0)),
    ])
}
fn terminal(initial: bool) -> Rc<Terminal> {
    let options = Rc::new(OptionsService::new(
        false,
        [
            ("cols", OptionValue::Number(5.0)),
            ("rows", OptionValue::Number(3.0)),
            (
                "windowsPty",
                if initial {
                    pty()
                } else {
                    OptionValue::object(())
                },
            ),
        ],
    ));
    Rc::new(Terminal::with_options(options).unwrap())
}
fn set(t: &Terminal, value: OptionValue) {
    let options = t.with_state(|s| Rc::clone(&s.options)).unwrap();
    options.set("windowsPty", value).unwrap();
}
fn wrapped(t: &Terminal) -> bool {
    t.with_state(|s| {
        let active = s.active_buffer();
        let buffer = active.borrow();
        let line = buffer.lines.get(1).unwrap();
        let wrapped = line.borrow().is_wrapped;
        wrapped
    })
    .unwrap()
}
fn observe(t: &Terminal, trace: &Trace, label: &str) {
    trace.borrow_mut().push(format!("{label}:{}", wrapped(t)));
}
fn listen(t: &Rc<Terminal>, trace: &Trace, label: &'static str) {
    let target = Rc::downgrade(t);
    let trace = Rc::clone(trace);
    t.on_line_feed()
        .subscribe(move |_| observe(&target.upgrade().unwrap(), &trace, label));
}
fn verify(t: &Terminal, trace: &Trace, name: &str) {
    observe(t, trace, "final");
    let expected = include_str!("../../../results/windows-wrapping/source.tsv")
        .lines()
        .find_map(|row| {
            let (key, value) = row.split_once('\t').unwrap();
            (key == name).then_some(value)
        })
        .unwrap();
    assert_eq!(trace.borrow().join(","), expected, "{name}");
    t.dispose().unwrap();
}
#[test]
fn linefeed_registration_and_toggle_order_matches_original() {
    for name in [
        "lf-before-enable",
        "lf-constructor-only",
        "lf-enabled-idempotent",
        "lf-reenabled",
    ] {
        let t = terminal(name == "lf-constructor-only");
        let trace = Trace::default();
        match name {
            "lf-before-enable" => {
                listen(&t, &trace, "early");
                set(&t, pty());
                listen(&t, &trace, "late");
            }
            "lf-constructor-only" => listen(&t, &trace, "user"),
            "lf-enabled-idempotent" => {
                set(&t, pty());
                listen(&t, &trace, "user");
                set(&t, pty());
            }
            "lf-reenabled" => {
                set(&t, pty());
                listen(&t, &trace, "early");
                set(&t, OptionValue::object(()));
                set(&t, pty());
                listen(&t, &trace, "late");
            }
            _ => unreachable!(),
        }
        t.write("abcde\n").unwrap();
        verify(&t, &trace, name);
    }
}
#[test]
fn changing_policy_inside_linefeed_keeps_snapshot_traversal() {
    for enable in [false, true] {
        let t = terminal(false);
        let trace = Trace::default();
        let target = Rc::downgrade(&t);
        let output = Rc::clone(&trace);
        t.on_line_feed().subscribe(move |_| {
            let t = target.upgrade().unwrap();
            observe(&t, &output, "early");
            set(
                &t,
                if enable {
                    pty()
                } else {
                    OptionValue::object(())
                },
            );
        });
        if !enable {
            set(&t, pty());
        }
        listen(&t, &trace, "late");
        t.write("abcde\n").unwrap();
        verify(
            &t,
            &trace,
            if enable {
                "lf-enable-during-dispatch"
            } else {
                "lf-disable-during-dispatch"
            },
        );
    }
}
#[test]
fn linefeed_error_stops_before_later_workaround() {
    let t = terminal(false);
    let trace = Trace::default();
    let output = Rc::clone(&trace);
    let target = Rc::downgrade(&t);
    t.on_line_feed().try_subscribe(move |_| {
        observe(&target.upgrade().unwrap(), &output, "early");
        Err(EmitError::callback(std::io::Error::other(
            "listener failed",
        )))
    });
    set(&t, pty());
    let error = t.write("abcde\n").unwrap_err();
    let mut cause: &dyn std::error::Error = &error;
    while let Some(source) = cause.source() {
        cause = source;
    }
    assert_eq!(cause.to_string(), "listener failed");
    trace.borrow_mut().push(format!("error:{cause}"));
    verify(&t, &trace, "lf-error-before-hook");
}
#[test]
fn disposal_removes_hook_before_clearing_the_active_emitter() {
    for before in [true, false] {
        let t = terminal(false);
        let trace = Trace::default();
        let register = || {
            let target = Rc::downgrade(&t);
            let output = Rc::clone(&trace);
            t.on_line_feed().subscribe(move |_| {
                let t = target.upgrade().unwrap();
                observe(&t, &output, "dispose");
                t.dispose().unwrap();
            });
        };
        if before {
            register();
        }
        set(&t, pty());
        if !before {
            register();
        }
        listen(&t, &trace, "late");
        t.write("abcde\n").unwrap();
        verify(
            &t,
            &trace,
            if before {
                "lf-dispose-before-hook"
            } else {
                "lf-dispose-after-hook"
            },
        );
    }
}
fn csi(t: &Rc<Terminal>, trace: &Trace, consume: bool) {
    let target = Rc::downgrade(t);
    let trace = Rc::clone(trace);
    t.with_parser(|parser| {
        parser.register_csi_handler(FunctionIdentifier::new("H"), move |_| {
            observe(&target.upgrade().unwrap(), &trace, "user");
            HandlerResult::Ready(consume)
        })
    })
    .unwrap()
    .unwrap();
}
#[test]
fn csi_priority_consumption_and_reenabled_order_match_original() {
    for before in [true, false] {
        for consume in [true, false] {
            let t = terminal(false);
            t.write("abcde\x1b[2;1H").unwrap();
            let trace = Trace::default();
            if before {
                csi(&t, &trace, consume);
            }
            set(&t, pty());
            if !before {
                csi(&t, &trace, consume);
            }
            t.write("\x1b[3;1H").unwrap();
            verify(
                &t,
                &trace,
                &format!(
                    "csi-{}-{}",
                    if before { "before" } else { "after" },
                    if consume { "consume" } else { "bubble" }
                ),
            );
        }
    }
    let t = terminal(false);
    t.write("abcde\x1b[2;1H").unwrap();
    let trace = Trace::default();
    set(&t, pty());
    csi(&t, &trace, true);
    set(&t, OptionValue::object(()));
    set(&t, pty());
    t.write("\x1b[3;1H").unwrap();
    verify(&t, &trace, "csi-reenabled");
}
#[test]
fn reset_rechecks_mutated_windows_pty_object() {
    let t = terminal(false);
    let build = Rc::new(Cell::new(21375.0));
    let properties = Rc::clone(&build);
    set(
        &t,
        OptionValue::Object(OptionObject::with_properties(
            properties,
            |_| Ok(OptionValue::from("[object Object]")),
            |build, key| {
                Ok(match key {
                    "backend" => OptionValue::from("conpty"),
                    "buildNumber" => OptionValue::Number(build.get()),
                    _ => OptionValue::Undefined,
                })
            },
        )),
    );
    build.set(21376.0);
    t.reset().unwrap();
    let trace = Trace::default();
    listen(&t, &trace, "user");
    t.write("abcde\n").unwrap();
    verify(&t, &trace, "reset-rechecks-mutated-object");
}
