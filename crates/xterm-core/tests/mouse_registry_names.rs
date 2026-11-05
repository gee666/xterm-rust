// Copyright (c) 2019, 2026 The xterm.js authors. All rights reserved. MIT license.
// Names and expected order come from unchanged MouseStateService Object.keys.
use std::rc::Rc;
use xterm_core::{
    lifecycle::Disposable,
    mouse_state::{MouseEventTypes, MouseProtocol, MouseStateService},
};

type Service = MouseStateService<()>;

fn replay(case: &str) {
    let mut service = Service::new();
    let mut current = "";
    let mut snapshots = 0;
    for line in include_str!("fixtures/mouse-registry-names.tsv").lines() {
        let mut fields = line.split('\t');
        let operation = fields.next().unwrap();
        if operation == "case" {
            current = fields.next().unwrap();
            service = Service::new();
        } else if current == case {
            match operation {
                "add" => {
                    let name = fields.next().unwrap();
                    service.add_protocol(
                        name,
                        MouseProtocol {
                            events: MouseEventTypes::DOWN,
                            restrict: Rc::new(|_| Ok(true)),
                        },
                    );
                    service.add_encoding(name, Rc::new(|_| Ok(Vec::new())));
                }
                "protocols" | "encodings" => {
                    let actual = if operation == "protocols" {
                        service.protocol_names()
                    } else {
                        service.encoding_names()
                    };
                    assert_eq!(actual, fields.collect::<Vec<_>>(), "{case}: {operation}");
                    snapshots += 1;
                }
                "reset" => service.reset().unwrap(),
                "dispose" => service.dispose(),
                _ => panic!("unknown registry fixture operation {operation:?}"),
            }
        }
    }
    assert!(snapshots > 0, "missing registry fixture case {case}");
}

#[test]
fn canonical_numeric_boundaries_match_source_object_keys() {
    replay("canonical-boundaries");
}

#[test]
fn replacements_preserve_named_insertion_order_and_numeric_order() {
    replay("replacement-order");
}

#[test]
fn seeded_mixed_names_match_source_before_and_after_reset_disposal() {
    replay("seeded-mixed-and-replaced");
}
