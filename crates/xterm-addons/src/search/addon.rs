// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{
    DecorationManager, NumericHighlightLimit, SearchAddonError, SearchDecorationOptionsHandle,
    SearchHighlightLimit, SearchHost, SearchOptions, SearchResultTracker, SearchTermHandle,
};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
use xterm_core::{
    event::{EmitError, Emitter, Event},
    lifecycle::{Disposable, DisposableHandle},
    search::SearchLineCache,
    terminal::TerminalEvent,
};
#[path = "completion.rs"]
mod completion;
#[path = "find.rs"]
mod find;

pub trait SearchOptionsSource {
    /// Host adapters use their retained raw values for strict flag equality.
    fn as_any(&self) -> &dyn std::any::Any;
    fn read(&self) -> Result<SearchOptions, SearchAddonError>;
    /// Raw request truthiness, without invoking getters or coercion callbacks.
    /// Typed Rust option records are always truthy.
    fn is_truthy(&self) -> bool {
        true
    }
    /// Copy the live request for delayed refresh and force incremental matching.
    /// Browser adapters preserve own enumerable properties and accessor order.
    fn for_refresh(&self) -> Result<SearchOptionsHandle, SearchAddonError> {
        let mut options = self.read()?;
        options.matching.incremental = Some(true);
        Ok(Rc::new(RefCell::new(options)))
    }
    /// Read only decoration eligibility, without inspecting matching flags.
    fn decorations(&self) -> Result<Option<SearchDecorationOptionsHandle>, SearchAddonError> {
        Ok(self.read()?.decorations)
    }
    /// Compare stored then incoming flags, resolving the live stored request
    /// again before each property. Either getter may replace it by reentry.
    /// Native records have no per-property accessors; host adapters preserve them.
    fn flags_changed(
        &self,
        stored: &dyn Fn() -> Option<SearchOptionsHandle>,
    ) -> Result<bool, SearchAddonError> {
        for index in 0..3 {
            let left = stored()
                .ok_or(SearchAddonError::MissingOptions)?
                .read()?
                .matching;
            let left = [left.case_sensitive, left.regex, left.whole_word][index];
            let right = self.read()?.matching;
            let right = [right.case_sensitive, right.regex, right.whole_word][index];
            if left != right {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
impl SearchOptionsSource for RefCell<SearchOptions> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        self.try_borrow()
            .map(|options| options.clone())
            .map_err(|_| SearchAddonError::Borrowed)
    }
}
pub type SearchOptionsHandle = Rc<dyn SearchOptionsSource>;
const DEFAULT_HIGHLIGHT_LIMIT: f64 = 1000.0;
const REFRESH_DELAY_MS: u64 = 200;

pub(super) struct Inner {
    host: RefCell<Option<Rc<dyn SearchHost>>>,
    manager: RefCell<Option<Rc<DecorationManager>>>,
    cached_term: RefCell<Option<SearchTermHandle>>,
    last_options: RefCell<Option<SearchOptionsHandle>>,
    cache: RefCell<SearchLineCache>,
    tracker: SearchResultTracker,
    highlight_limit: Rc<dyn SearchHighlightLimit>,
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
#[derive(Clone)]
pub struct WeakSearchAddon {
    inner: Weak<Inner>,
}
impl WeakSearchAddon {
    pub fn upgrade(&self) -> Option<SearchAddon> {
        self.inner.upgrade().map(|inner| SearchAddon { inner })
    }
}
impl Default for SearchAddon {
    fn default() -> Self {
        Self::new(None)
    }
}
impl SearchAddon {
    pub fn downgrade(&self) -> WeakSearchAddon {
        WeakSearchAddon {
            inner: Rc::downgrade(&self.inner),
        }
    }
    pub fn new(highlight_limit: Option<f64>) -> Self {
        Self::with_limit(Rc::new(NumericHighlightLimit(
            highlight_limit.unwrap_or(DEFAULT_HIGHLIGHT_LIMIT),
        )))
    }
    pub fn with_limit(highlight_limit: Rc<dyn SearchHighlightLimit>) -> Self {
        Self {
            inner: Rc::new(Inner {
                host: RefCell::new(None),
                manager: RefCell::new(None),
                cached_term: RefCell::new(None),
                last_options: RefCell::new(None),
                cache: RefCell::new(SearchLineCache::default()),
                tracker: SearchResultTracker::default(),
                highlight_limit,
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
            self.inner.cached_term.borrow_mut().take();
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
        let term = self.inner.cached_term.borrow().clone();
        let options = self.inner.last_options.borrow().clone();
        if term.as_ref().is_some_and(|term| term.is_truthy())
            && options
                .map(|options| options.decorations())
                .transpose()?
                .flatten()
                .is_some()
        {
            let weak = Rc::downgrade(&self.inner);
            let timer = self.host()?.timeout(
                REFRESH_DELAY_MS,
                Rc::new(move || {
                    if let Some(inner) = weak.upgrade() {
                        let addon = SearchAddon { inner };
                        let term = addon
                            .inner
                            .cached_term
                            .borrow()
                            .clone()
                            .unwrap_or_else(|| Rc::new(super::term::NativeTerm::Undefined));
                        // Source clears the cache before spreading options. A
                        // getter may reenter or throw while copying the request.
                        addon.inner.cached_term.borrow_mut().take();
                        let previous = addon.inner.last_options.borrow().clone();
                        let options = previous
                            .map(|previous| previous.for_refresh())
                            .transpose()?;
                        addon.find_term_with_scroll(term, options, true, || Ok(true))?;
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
