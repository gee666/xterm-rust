// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use std::{
    any::Any,
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};
use xterm_addons::search::{
    SearchAddon, SearchAddonError, SearchDecorationOptions, SearchHost, SearchOptions,
    SearchOptionsHandle, TimeoutCallback,
};
use xterm_core::{
    decoration::DecorationService,
    event::EmitError,
    headless::Terminal,
    lifecycle::{to_disposable, DisposableHandle},
    search::{SearchResult, SearchSelection},
};
use xterm_web::{
    search_host::SearchTerminal,
    selection::service::{SelectionPlatform, SelectionService},
};

type Job = (u64, TimeoutCallback);
#[derive(Default)]
pub struct Clock {
    pub now: Cell<u64>,
    next: Cell<u64>,
    jobs: RefCell<BTreeMap<u64, Job>>,
}
impl Clock {
    pub fn pending(&self) -> usize {
        self.jobs.borrow().len()
    }
    pub fn advance(&self, now: u64) -> Result<(), SearchAddonError> {
        assert!(now >= self.now.get());
        for _ in 0..1000 {
            let next = self
                .jobs
                .borrow()
                .iter()
                .filter(|(_, (due, _))| *due <= now)
                .min_by_key(|(id, (due, _))| (*due, **id))
                .map(|(&id, (due, _))| (id, *due));
            let Some((id, due)) = next else {
                self.now.set(now);
                return Ok(());
            };
            let (_, callback) = self.jobs.borrow_mut().remove(&id).unwrap();
            self.now.set(due);
            callback()?;
        }
        panic!("virtual search timer did not quiesce within 1000 callbacks");
    }
}
#[derive(Default)]
pub struct Element {
    pub classes: RefCell<BTreeSet<String>>,
    pub outline: RefCell<Option<String>>,
}
pub struct Host {
    pub terminal: SearchTerminal,
    pub clock: Rc<Clock>,
}
impl SearchHost for Host {
    fn terminal(&self) -> Rc<Terminal> {
        Rc::clone(&self.terminal.native)
    }
    fn decorations(&self) -> Rc<DecorationService> {
        Rc::clone(&self.terminal.decorations)
    }
    fn selection(&self) -> Result<Option<SearchSelection>, SearchAddonError> {
        self.terminal.selection_position()
    }
    fn clear_selection(&self) -> Result<(), SearchAddonError> {
        self.terminal.clear_selection()
    }
    fn select(&self, result: &SearchResult) -> Result<(), SearchAddonError> {
        self.terminal.select(result)
    }
    fn style_decoration(
        &self,
        element: &Rc<dyn Any>,
        border: Option<&str>,
    ) -> Result<(), EmitError> {
        let element = element.downcast_ref::<Element>().unwrap();
        if element
            .classes
            .borrow_mut()
            .insert("xterm-find-result-decoration".into())
        {
            if let Some(border) = border.filter(|border| !border.is_empty()) {
                *element.outline.borrow_mut() = Some(format!("1px solid {border}"));
            }
        }
        Ok(())
    }
    fn now_ms(&self) -> u64 {
        self.clock.now.get()
    }
    fn timeout(
        &self,
        delay: u64,
        callback: TimeoutCallback,
    ) -> Result<DisposableHandle, SearchAddonError> {
        let id = self.clock.next.get();
        self.clock.next.set(id + 1);
        self.clock
            .jobs
            .borrow_mut()
            .insert(id, (self.clock.now.get() + delay, callback));
        let clock = Rc::downgrade(&self.clock);
        Ok(to_disposable(move || {
            if let Some(clock) = clock.upgrade() {
                clock.jobs.borrow_mut().remove(&id);
            }
        }))
    }
}
pub struct Harness {
    pub addon: SearchAddon,
    pub host: Rc<Host>,
}
impl Harness {
    pub fn new(cols: usize, rows: usize, limit: Option<f64>) -> Self {
        let native = Rc::new(Terminal::new(cols, rows, 1000).unwrap());
        let selection = Rc::new(SelectionService::new(SelectionPlatform::default()));
        let buffers = native
            .with_state(|state| Rc::clone(&state.buffers))
            .unwrap();
        let decorations = Rc::new(DecorationService::with_buffers(&buffers).unwrap());
        let host = Rc::new(Host {
            terminal: SearchTerminal {
                native,
                selection,
                decorations,
            },
            clock: Rc::new(Clock::default()),
        });
        let addon = SearchAddon::new(limit);
        addon.activate(host.clone()).unwrap();
        Self { addon, host }
    }
    pub fn write(&self, value: &str) {
        self.host.terminal.native.write(value).unwrap();
    }
    pub fn selection(&self) -> Option<SearchSelection> {
        self.host.terminal.selection_position().unwrap()
    }
    pub fn selected_text(&self) -> Vec<u16> {
        let active = self
            .host
            .terminal
            .native
            .with_state(|state| state.active_buffer())
            .unwrap();
        let buffer = active.borrow();
        self.host
            .terminal
            .selection
            .selection_text(&buffer, self.host.terminal.dimensions().unwrap())
            .unwrap()
    }
    pub fn decorations(&self) -> Vec<xterm_core::decoration::Decoration> {
        self.host
            .terminal
            .decorations
            .decorations()
            .unwrap()
            .collect()
    }
}
pub fn term(value: &str) -> Vec<u16> {
    value.encode_utf16().collect()
}
pub fn options() -> SearchOptionsHandle {
    Rc::new(RefCell::new(SearchOptions {
        decorations: Some(Rc::new(SearchDecorationOptions {
            match_background: Some("#112233".into()),
            match_border: Some("#223344".into()),
            match_overview_ruler: "#ff0000".into(),
            active_match_background: Some("#334455".into()),
            active_match_border: Some("#445566".into()),
            active_match_overview_ruler: "#00ff00".into(),
        })),
        ..SearchOptions::default()
    }))
}
