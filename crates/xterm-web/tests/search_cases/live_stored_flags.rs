// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use xterm_addons::search::{SearchDecorationOptionsHandle, SearchOptionsSource};

type Read = dyn Fn() -> Result<SearchOptions, SearchAddonError>;
struct Request {
    read: Box<Read>,
    decorate: Box<dyn Fn() -> Result<(), SearchAddonError>>,
    decorated: bool,
}
impl SearchOptionsSource for Request {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        (self.read)()
    }
    fn decorations(&self) -> Result<Option<SearchDecorationOptionsHandle>, SearchAddonError> {
        (self.decorate)()?;
        Ok(if self.decorated {
            options().read()?.decorations
        } else {
            None
        })
    }
}
fn flags(values: [Option<bool>; 3]) -> SearchOptions {
    SearchOptions {
        matching: xterm_core::search::SearchOptions {
            case_sensitive: values[0],
            regex: values[1],
            whole_word: values[2],
            ..Default::default()
        },
        ..Default::default()
    }
}
fn find(
    h: &Harness,
    reverse: bool,
    request: SearchOptionsHandle,
) -> Result<bool, SearchAddonError> {
    if reverse {
        h.addon.find_previous(&term("foo"), Some(request))
    } else {
        h.addon.find_next(&term("foo"), Some(request))
    }
}

#[test]
fn stored_flags_are_read_before_incoming_and_stop_on_first_difference() {
    for reverse in [false, true] {
        for changed in [None, Some(0), Some(1), Some(2)] {
            let h = Harness::new(20, 3, None);
            h.write("foo FOO foo");
            h.addon.find_next(&term("foo"), None).unwrap();
            let armed = Rc::new(Cell::new(false));
            let trace = Rc::new(RefCell::new(Vec::new()));
            let stored: SearchOptionsHandle = Rc::new(Request {
                read: Box::new({
                    let armed = armed.clone();
                    let trace = trace.clone();
                    move || {
                        if armed.get() {
                            trace.borrow_mut().push("stored");
                        }
                        Ok(SearchOptions::default())
                    }
                }),
                decorate: Box::new(|| Ok(())),
                decorated: false,
            });
            let reads = Rc::new(Cell::new(0));
            let request = Rc::new(Request {
                read: Box::new({
                    let trace = trace.clone();
                    let reads = reads.clone();
                    move || {
                        let count = reads.get();
                        reads.set(count + 1);
                        trace.borrow_mut().push("incoming");
                        if count > changed.unwrap_or(2) {
                            return Err(SearchAddonError::InvalidCoordinates);
                        }
                        let mut value = [None; 3];
                        if let Some(index) = changed {
                            value[index] = Some(false);
                        }
                        Ok(flags(value))
                    }
                }),
                decorate: Box::new({
                    let addon = h.addon.downgrade();
                    move || {
                        if !armed.get() {
                            addon
                                .upgrade()
                                .unwrap()
                                .find_next(&term("foo"), Some(stored.clone()))?;
                            armed.set(true);
                        }
                        Ok(())
                    }
                }),
                decorated: true,
            });
            assert!(matches!(
                find(&h, reverse, request),
                Err(SearchAddonError::InvalidCoordinates)
            ));
            let mut expected = Vec::new();
            for _ in 0..=changed.unwrap_or(2) {
                expected.extend(["stored", "incoming"]);
            }
            expected.push("incoming");
            assert_eq!(*trace.borrow(), expected);
            assert_eq!(h.selection().is_some(), changed.is_some());
        }
    }
}

#[test]
fn either_operand_can_replace_live_stored_request_before_next_property() {
    for reverse in [false, true] {
        for side in ["stored", "incoming"] {
            for at in [0, 1] {
                let h = Harness::new(20, 3, None);
                h.write("foo FOO foo");
                h.addon.find_next(&term("foo"), None).unwrap();
                let armed = Rc::new(Cell::new(false));
                let trace = Rc::new(RefCell::new(Vec::new()));
                let make_read = |name| -> Box<Read> {
                    let reads = Cell::new(0);
                    let armed = armed.clone();
                    let trace = trace.clone();
                    let addon = h.addon.downgrade();
                    Box::new(move || {
                        if armed.get() {
                            trace.borrow_mut().push(name);
                            let count = reads.get();
                            reads.set(count + 1);
                            if name == side && count == at {
                                // Missing options are legal for the nested search,
                                // but the next stored property access must fail.
                                armed.set(false);
                                let result = addon.upgrade().unwrap().find_next(&term("foo"), None);
                                armed.set(true);
                                result?;
                            }
                        }
                        Ok(SearchOptions::default())
                    })
                };
                let stored: SearchOptionsHandle = Rc::new(Request {
                    read: make_read("stored"),
                    decorate: Box::new(|| Ok(())),
                    decorated: false,
                });
                let request = Rc::new(Request {
                    read: make_read("incoming"),
                    decorate: Box::new({
                        let addon = h.addon.downgrade();
                        move || {
                            if !armed.get() {
                                addon
                                    .upgrade()
                                    .unwrap()
                                    .find_next(&term("foo"), Some(stored.clone()))?;
                                armed.set(true);
                            }
                            Ok(())
                        }
                    }),
                    decorated: true,
                });
                assert!(matches!(
                    find(&h, reverse, request),
                    Err(SearchAddonError::MissingOptions)
                ));
                let expected: Vec<_> = (0..=at).flat_map(|_| ["stored", "incoming"]).collect();
                assert_eq!(*trace.borrow(), expected);
                assert!(h.selection().is_some());
            }
        }
    }
}
