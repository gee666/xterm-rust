// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{DecorationManager, SearchAddonError, SearchHost, SearchOptions, SearchResultTracker};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::{
    event::{EmitError, Emitter, Event},
    lifecycle::{Disposable, DisposableHandle},
    search::{SearchLineCache, SearchState},
    terminal::TerminalEvent,
};
#[path = "find.rs"]
mod find;

pub type SearchOptionsHandle = Rc<RefCell<SearchOptions>>;
const DEFAULT_HIGHLIGHT_LIMIT: f64 = 1000.0;
const REFRESH_DELAY_MS: u64 = 200;

pub(super) struct Inner {
    host: RefCell<Option<Rc<dyn SearchHost>>>,
    manager: RefCell<Option<Rc<DecorationManager>>>,
    state: RefCell<SearchState>,
    last_options: RefCell<Option<SearchOptionsHandle>>,
    cache: RefCell<SearchLineCache>,
    tracker: SearchResultTracker,
    highlight_limit: f64,
    highlight_timeout: RefCell<Option<DisposableHandle>>,
    cache_timeout: RefCell<Option<DisposableHandle>>,
    cache_listener: RefCell<Option<DisposableHandle>>,
    listeners: RefCell<Vec<DisposableHandle>>,
    before: Emitter<()>,
    after: Emitter<()>,
    disposed: Cell<bool>,
}

/// Clones refer to one addon, preserving event and selected-decoration identity.
#[derive(Clone)]
pub struct SearchAddon {
    inner: Rc<Inner>,
}
impl Default for SearchAddon {
    fn default() -> Self {
        Self::new(None)
    }
}
impl SearchAddon {
    pub fn new(highlight_limit: Option<f64>) -> Self {
        Self {
            inner: Rc::new(Inner {
                host: RefCell::new(None),
                manager: RefCell::new(None),
                state: RefCell::new(SearchState::default()),
                last_options: RefCell::new(None),
                cache: RefCell::new(SearchLineCache::default()),
                tracker: SearchResultTracker::default(),
                highlight_limit: highlight_limit.unwrap_or(DEFAULT_HIGHLIGHT_LIMIT),
                highlight_timeout: RefCell::new(None),
                cache_timeout: RefCell::new(None),
                cache_listener: RefCell::new(None),
                listeners: RefCell::new(Vec::new()),
                before: Emitter::new(),
                after: Emitter::new(),
                disposed: Cell::new(false),
            }),
        }
    }
    pub fn activate(&self, host: Rc<dyn SearchHost>) -> Result<(), SearchAddonError> {
        self.invalidate_cache();
        *self.inner.host.borrow_mut() = Some(Rc::clone(&host));
        *self.inner.manager.borrow_mut() = Some(Rc::new(DecorationManager::new(Rc::clone(&host))));
        let weak = Rc::downgrade(&self.inner);
        let listener = host.terminal().on_event().try_subscribe(move |event| {
            if matches!(
                event,
                TerminalEvent::WriteParsed | TerminalEvent::Resize { .. }
            ) {
                if let Some(inner) = weak.upgrade() {
                    SearchAddon { inner }
                        .update_matches()
                        .map_err(EmitError::callback)?;
                }
            }
            Ok(())
        });
        if self.inner.disposed.get() {
            listener.dispose();
        } else {
            self.inner.listeners.borrow_mut().push(listener);
        }
        Ok(())
    }
    pub fn on_before_search(&self) -> Event<()> {
        self.inner.before.event()
    }
    pub fn on_after_search(&self) -> Event<()> {
        self.inner.after.event()
    }
    pub fn on_did_change_results(&self) -> Event<super::SearchResultChange> {
        self.inner.tracker.on_did_change_results()
    }
    pub fn clear_active_decoration(&self) -> Result<(), SearchAddonError> {
        self.inner.tracker.clear_selected()?;
        Ok(())
    }
    pub fn clear_decorations(&self, retain_cached_term: bool) -> Result<(), SearchAddonError> {
        self.inner.tracker.clear_selected()?;
        let manager = self.inner.manager.borrow().clone();
        if let Some(manager) = manager {
            manager.clear_highlights()?;
        }
        self.inner.tracker.clear_results();
        if !retain_cached_term {
            self.inner.state.borrow_mut().clear_cached_term();
        }
        Ok(())
    }
    pub fn dispose(&self) -> Result<(), SearchAddonError> {
        if self.inner.disposed.replace(true) {
            return Ok(());
        }
        Self::cancel(&self.inner.highlight_timeout);
        self.invalidate_cache();
        let listeners = self.inner.listeners.take();
        for listener in listeners {
            listener.dispose();
        }
        self.inner.tracker.dispose();
        self.inner.after.dispose();
        self.inner.before.dispose();
        self.clear_decorations(false)
    }
    fn host(&self) -> Result<Rc<dyn SearchHost>, SearchAddonError> {
        self.inner
            .host
            .borrow()
            .clone()
            .ok_or(SearchAddonError::NotLoaded)
    }
    fn cancel(slot: &RefCell<Option<DisposableHandle>>) {
        let handle = slot.borrow_mut().take();
        if let Some(handle) = handle {
            handle.dispose();
        }
    }
    fn invalidate_cache(&self) {
        self.inner.cache.borrow_mut().invalidate();
        Self::cancel(&self.inner.cache_timeout);
        Self::cancel(&self.inner.cache_listener);
    }
    fn init_cache(&self) -> Result<(), SearchAddonError> {
        let host = self.host()?;
        self.inner.cache.borrow_mut().init(host.now_ms());
        if self.inner.disposed.get() {
            return Ok(());
        }
        if self.inner.cache_listener.borrow().is_none() {
            let weak = Rc::downgrade(&self.inner);
            let listener = host.terminal().on_event().try_subscribe(move |event| {
                if matches!(
                    event,
                    TerminalEvent::LineFeed
                        | TerminalEvent::CursorMove
                        | TerminalEvent::Resize { .. }
                ) {
                    if let Some(inner) = weak.upgrade() {
                        SearchAddon { inner }.invalidate_cache();
                    }
                }
                Ok(())
            });
            *self.inner.cache_listener.borrow_mut() = Some(listener);
        }
        if self.inner.cache_timeout.borrow().is_none() {
            self.schedule_cache_timeout()?;
        }
        Ok(())
    }
    fn schedule_cache_timeout(&self) -> Result<(), SearchAddonError> {
        let host = self.host()?;
        let deadline = self.inner.cache.borrow().deadline_ms();
        if let Some(deadline) = deadline {
            let weak = Rc::downgrade(&self.inner);
            let timer = host.timeout(
                deadline.saturating_sub(host.now_ms()),
                Rc::new(move || {
                    if let Some(inner) = weak.upgrade() {
                        let addon = SearchAddon { inner };
                        Self::cancel(&addon.inner.cache_timeout);
                        let host = addon.host()?;
                        addon.inner.cache.borrow_mut().expire(host.now_ms());
                        if addon.inner.cache.borrow().deadline_ms().is_none() {
                            addon.invalidate_cache();
                        } else {
                            addon.schedule_cache_timeout()?;
                        }
                    }
                    Ok(())
                }),
            )?;
            *self.inner.cache_timeout.borrow_mut() = Some(timer);
        }
        Ok(())
    }
    fn update_matches(&self) -> Result<(), SearchAddonError> {
        Self::cancel(&self.inner.highlight_timeout);
        let term = self.inner.state.borrow().cached_search_term.clone();
        let options = self.inner.last_options.borrow().clone();
        if term.as_ref().is_some_and(|term| !term.is_empty())
            && options
                .as_ref()
                .is_some_and(|options| options.borrow().decorations.is_some())
        {
            let weak = Rc::downgrade(&self.inner);
            let timer = self.host()?.timeout(
                REFRESH_DELAY_MS,
                Rc::new(move || {
                    if let Some(inner) = weak.upgrade() {
                        let addon = SearchAddon { inner };
                        let term = addon
                            .inner
                            .state
                            .borrow()
                            .cached_search_term
                            .clone()
                            .unwrap_or_default();
                        let previous = addon.inner.last_options.borrow().clone();
                        let options = previous.map(|previous| {
                            let mut options = previous.borrow().clone();
                            options.matching.incremental = Some(true);
                            Rc::new(RefCell::new(options))
                        });
                        addon.inner.state.borrow_mut().clear_cached_term();
                        addon.find_previous_internal(&term, options, true)?;
                    }
                    Ok(())
                }),
            )?;
            if self.inner.disposed.get() {
                timer.dispose();
            } else {
                *self.inner.highlight_timeout.borrow_mut() = Some(timer);
            }
        }
        Ok(())
    }
}
