// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use crate::projection;
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::marker::Marker,
    headless::{HeadlessError, Terminal},
    lifecycle::DisposableHandle,
    options::OptionValue,
    public_api::addon_manager::AddonHandle,
    search::{with_buffer, SearchEngine, SearchLineCache, SearchOptions},
    serialize::{SerializeAddon, SerializeOptions},
};

pub struct Harness {
    pub terminal: Rc<Terminal>,
    pub addon: Rc<SerializeAddon>,
    registration: AddonHandle<Terminal, HeadlessError>,
    pub markers: Vec<Marker>,
    subscriptions: Vec<DisposableHandle>,
    pub events: Rc<RefCell<Vec<String>>>,
    pub marker_events: Rc<RefCell<Vec<usize>>>,
}
impl Harness {
    pub fn new(cols: usize, rows: usize, scrollback: usize) -> Self {
        let terminal = Rc::new(Terminal::new(cols, rows, scrollback).unwrap());
        terminal
            .options()
            .set("allowProposedApi", OptionValue::Bool(true))
            .unwrap();
        let addon = Rc::new(SerializeAddon::new());
        let registration = terminal.load_addon(addon.clone()).unwrap();
        let events = Rc::new(RefCell::new(Vec::new()));
        let output = events.clone();
        let subscription = terminal.on_event().subscribe(move |event| {
            if let Some(event) = projection::event(event) {
                output.borrow_mut().push(event);
            }
        });
        Self {
            terminal,
            addon,
            registration,
            markers: Vec::new(),
            subscriptions: vec![subscription],
            events,
            marker_events: Rc::new(RefCell::new(Vec::new())),
        }
    }
    pub fn add_marker(&mut self, offset: i64, alias: bool) -> Result<String, HeadlessError> {
        let marker = if alias {
            self.terminal.add_marker(offset)?
        } else {
            self.terminal.register_marker(offset)?
        };
        assert!(
            !self
                .markers
                .iter()
                .any(|previous| previous.id() == marker.id()),
            "marker ID reused"
        );
        let ordinal = self.markers.len();
        let output = self.marker_events.clone();
        self.subscriptions.push(
            marker
                .on_dispose()
                .subscribe(move |_| output.borrow_mut().push(ordinal)),
        );
        self.markers.push(marker);
        Ok(ordinal.to_string())
    }
    pub fn cursor_line(&self) -> Option<xterm_core::buffer::LineRef> {
        self.terminal
            .with_state(|state| {
                let active = state.active_buffer();
                let buffer = active.borrow();
                buffer.lines.get(buffer.ybase + buffer.y)
            })
            .unwrap()
    }
    pub fn state(&self) -> String {
        self.terminal
            .with_state(projection::snapshot_state)
            .unwrap()
    }
    pub fn saved(&self) -> String {
        self.terminal
            .with_state(|state| {
                let normal = state.buffers.normal();
                let alternate = state.buffers.alt();
                let normal = normal.borrow();
                let alternate = alternate.borrow();
                format!(
                    "{},{},{},{}",
                    normal.saved_x, normal.saved_y, alternate.saved_x, alternate.saved_y
                )
            })
            .unwrap()
    }
    pub fn marker_state(&self) -> String {
        if self.markers.is_empty() {
            return "-".into();
        }
        self.markers
            .iter()
            .map(|m| format!("{},{}", m.line(), u8::from(m.is_disposed())))
            .collect::<Vec<_>>()
            .join("|")
    }
    pub fn public_markers(&self) -> String {
        let current = self.terminal.markers().unwrap();
        if current.is_empty() {
            return "-".into();
        }
        current
            .iter()
            .map(|marker| {
                self.markers
                    .iter()
                    .position(|m| m.ptr_eq(marker))
                    .map_or_else(|| "-1".into(), |index| index.to_string())
            })
            .collect::<Vec<_>>()
            .join(",")
    }
    pub fn serialized(&self) -> Result<Vec<u16>, String> {
        self.addon
            .serialize(&SerializeOptions::default())
            .map_err(|e| e.to_string())
    }
    pub fn search(&self) -> String {
        self.terminal
            .with_state(|state| {
                let active = state.active_buffer();
                let active = active.borrow();
                with_buffer(&active, state.cols(), |view| {
                    let mut cache = SearchLineCache::default();
                    let mut engine = SearchEngine::new(view, &mut cache, 0);
                    Ok(["abc", "中", "next", "line", "e\u{301}"]
                        .into_iter()
                        .map(|term| {
                            let term: Vec<_> = term.encode_utf16().collect();
                            match engine.find(&term, 0, 0, &SearchOptions::default()) {
                                Ok(Some(result)) => format!(
                                    "{}:{}:{}:{}",
                                    result.row,
                                    result.col,
                                    result.size,
                                    projection::hex(&result.term)
                                ),
                                Ok(None) => "-".into(),
                                Err(error) => format!(
                                    "!{}",
                                    projection::hex(
                                        &error.to_string().encode_utf16().collect::<Vec<_>>()
                                    )
                                ),
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("/"))
                })
                .unwrap_or_else(|error| format!("!view:{error}"))
            })
            .unwrap()
    }
    pub fn finish(&self) -> Result<(), HeadlessError> {
        self.terminal.dispose()?;
        assert!(
            self.markers.iter().all(Marker::is_disposed),
            "terminal disposal leaves live markers"
        );
        assert!(
            self.registration.is_disposed(),
            "terminal disposal leaves addon registration live"
        );
        Ok(())
    }
}
