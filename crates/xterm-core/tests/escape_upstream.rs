// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Migrated EscapeSequenceParser.test.ts assertions.
use std::{cell::RefCell, rc::Rc};
use xterm_core::parser::{
    apc::ApcAction,
    dcs::DcsAction,
    escape::{EscapeSequenceParser, FunctionIdentifier},
    handler::HandlerResult,
    osc::OscAction,
    params::{ParamValue as V, Params},
    table::{ParserState as S, TransitionTable},
};
use xterm_core::text_decoder::utf32_to_string;

#[path = "escape_upstream_support/basic_handlers.rs"]
mod basic_handlers;
#[path = "escape_upstream_support/examples.rs"]
mod examples;
#[path = "escape_upstream_support/identifiers.rs"]
mod identifiers;
#[path = "escape_upstream_support/parameters.rs"]
mod parameters;
#[path = "escape_upstream_support/registrations.rs"]
mod registrations;
#[path = "escape_upstream_support/states.rs"]
mod states;
#[path = "escape_upstream_support/strings.rs"]
mod strings;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Event {
    Print(String),
    Execute(u32),
    Esc(String),
    Csi(String, Vec<V>),
    Osc(String),
    DcsHook(Vec<V>),
    DcsPut(String),
    DcsEnd(bool),
    ApcStart,
    ApcPut(String),
    ApcEnd(bool),
}

struct Fixture {
    parser: EscapeSequenceParser,
    events: Rc<RefCell<Vec<Event>>>,
    osc_data: Rc<RefCell<Vec<u16>>>,
}

fn text(units: &[u16]) -> String {
    String::from_utf16(units).unwrap()
}
fn data(value: &str) -> Vec<u32> {
    value.chars().map(u32::from).collect()
}

impl Fixture {
    fn new() -> Self {
        let events = Rc::new(RefCell::new(Vec::new()));
        let osc_data = Rc::new(RefCell::new(Vec::new()));
        let mut parser = EscapeSequenceParser::new();
        let log = Rc::clone(&events);
        parser.set_print_handler(move |codes| {
            log.borrow_mut()
                .push(Event::Print(text(&utf32_to_string(codes))))
        });
        let log = Rc::clone(&events);
        parser
            .set_execute_handler_fallback(move |code| log.borrow_mut().push(Event::Execute(code)));
        let log = Rc::clone(&events);
        parser.set_esc_handler_fallback(move |id| {
            log.borrow_mut()
                .push(Event::Esc(EscapeSequenceParser::ident_to_string(id)))
        });
        let log = Rc::clone(&events);
        parser.set_csi_handler_fallback(move |id, params| {
            log.borrow_mut().push(Event::Csi(
                EscapeSequenceParser::ident_to_string(id),
                params.to_array(),
            ))
        });
        let log = Rc::clone(&events);
        let payload = Rc::clone(&osc_data);
        parser
            .osc
            .set_handler_fallback(move |id, action| match action {
                OscAction::Start => payload.borrow_mut().clear(),
                OscAction::Put(units) => payload.borrow_mut().extend(units),
                OscAction::End(success) => log.borrow_mut().push(Event::Osc(format!(
                    "{id};{}, success: {success}",
                    text(&payload.borrow())
                ))),
            });
        let log = Rc::clone(&events);
        parser.dcs.set_handler_fallback(move |_, action| {
            log.borrow_mut().push(match action {
                DcsAction::Hook(params) => Event::DcsHook(params.to_array()),
                DcsAction::Put(units) => Event::DcsPut(text(&units)),
                DcsAction::Unhook(success) => Event::DcsEnd(success),
            })
        });
        let log = Rc::clone(&events);
        parser.apc.set_handler_fallback(move |_, action| {
            log.borrow_mut().push(match action {
                ApcAction::Start => Event::ApcStart,
                ApcAction::Put(units) => Event::ApcPut(text(&units)),
                ApcAction::End(success) => Event::ApcEnd(success),
            })
        });
        Self {
            parser,
            events,
            osc_data,
        }
    }
    fn parse(&mut self, value: &str) {
        assert!(self.parser.parse(&data(value), None).unwrap().is_none());
    }
    fn one(&mut self, code: u32) {
        assert!(self.parser.parse(&[code], None).unwrap().is_none());
    }
    fn events(&self) -> Vec<Event> {
        self.events.borrow().clone()
    }
    fn clear(&self) {
        self.events.borrow_mut().clear();
    }
}
