//! Ligature font loading and lookup state, kept separate from addon lifecycle.
use crate::{ligature_host as host, local_fonts::LocalFonts, utf16::from_js_string};
use js_sys::{Array, JsString, Promise};
use std::{
    cell::RefCell,
    future::Future,
    rc::Rc,
    task::{Context, Poll, Waker},
};
use wasm_bindgen::{closure::Closure, JsValue};
use wasm_bindgen_futures::future_to_promise;
use xterm_addons::ligatures::font::{Font, FontError, FontOptions};

const SOURCE_FONT_CACHE_SIZE: usize = 100_000;
#[derive(Clone, Copy, PartialEq)]
enum LoadState {
    Unloaded,
    Loading,
    Loaded,
    Failed,
}
pub struct JoinState {
    name: JsValue,
    loading: LoadState,
    font: Option<Font>,
    error: JsValue,
}
impl Default for JoinState {
    fn default() -> Self {
        Self {
            name: JsValue::UNDEFINED,
            loading: LoadState::Unloaded,
            font: None,
            error: JsValue::UNDEFINED,
        }
    }
}
fn font_error(error: FontError) -> JsValue {
    match error {
        FontError::Bounds => js_sys::RangeError::new(&error.to_string()).into(),
        FontError::CacheEmptyKey => js_sys::TypeError::new(&error.to_string()).into(),
        _ => js_sys::Error::new(&error.to_string()).into(),
    }
}
fn eager<F: Future<Output = Result<JsValue, JsValue>> + 'static>(future: F) -> Promise {
    // JS async functions run synchronously until their first await. Start the
    // query before fallback matching, then let the normal executor own it.
    let mut future = Box::pin(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(Ok(value)) => Promise::resolve(&value),
        Poll::Ready(Err(error)) => Promise::reject(&error),
        Poll::Pending => future_to_promise(future),
    }
}
fn success(
    term: &JsValue,
    name: &JsValue,
    state: &Rc<RefCell<JoinState>>,
    font: Option<Font>,
) -> Result<(), JsValue> {
    if *name != host::family(term)? {
        return Ok(());
    }
    let refresh = font.is_some();
    {
        let mut state = state.borrow_mut();
        state.loading = LoadState::Loaded;
        state.font = font;
    }
    if refresh {
        host::refresh(term)?;
    }
    Ok(())
}
fn failure(
    term: &JsValue,
    name: &JsValue,
    state: &Rc<RefCell<JoinState>>,
    error: JsValue,
) -> Result<(), JsValue> {
    if *name != host::family(term)? {
        return Ok(());
    }
    state.borrow_mut().loading = LoadState::Failed;
    if host::debug(term)? {
        let previous = state.borrow().error.clone();
        host::failure(&previous)?;
    }
    let mut state = state.borrow_mut();
    state.font = None;
    state.error = error;
    Ok(())
}
fn load(
    term: JsValue,
    name: JsValue,
    state: Rc<RefCell<JoinState>>,
    catalog: LocalFonts,
) -> Result<(), JsValue> {
    let result = Rc::new(RefCell::new(None));
    let destination = Rc::clone(&result);
    let request = name.clone();
    let promise = eager(async move {
        let font = catalog
            .load_value(&request)
            .await?
            .map(|font| {
                Font::load_buffer(
                    &font.bytes,
                    FontOptions {
                        cache_size: SOURCE_FONT_CACHE_SIZE,
                    },
                )
                .map_err(font_error)
            })
            .transpose()?;
        *destination.borrow_mut() = Some(font);
        Ok(JsValue::UNDEFINED)
    });
    let target = term.clone();
    let expected = name.clone();
    let completed = Rc::clone(&state);
    let loaded = Closure::wrap(Box::new(move || {
        let font = result
            .borrow_mut()
            .take()
            .ok_or_else(|| js_sys::Error::new("Missing completed font load"))?;
        success(&target, &expected, &completed, font)
    }) as Box<dyn FnMut() -> Result<(), JsValue>>)
    .into_js_value();
    let failed = Closure::wrap(
        Box::new(move |error: JsValue| failure(&term, &name, &state, error))
            as Box<dyn FnMut(JsValue) -> Result<(), JsValue>>,
    )
    .into_js_value();
    // Native Promise reactions retain source then/catch ordering and propagate
    // a refresh or debug callback exception through the same catch route.
    host::finish_load(&promise, &loaded, &failed)
}
fn native_ranges(state: &mut JoinState, text: &[u16]) -> Result<Option<Vec<[i64; 2]>>, FontError> {
    if state.loading != LoadState::Loaded {
        return Ok(None);
    }
    let Some(font) = &mut state.font else {
        return Ok(None);
    };
    let ranges = font.find_ligature_ranges(text)?;
    let ranges = ranges.try_borrow().map_err(|_| FontError::Borrowed)?;
    let mut copied = Vec::new();
    copied.try_reserve_exact(ranges.len())?;
    copied.extend_from_slice(&ranges);
    Ok(Some(copied))
}
pub fn join(
    term: &JsValue,
    patterns: &JsValue,
    state: &Rc<RefCell<JoinState>>,
    catalog: &LocalFonts,
    text: &JsString,
) -> Result<Array, JsValue> {
    let family = host::family(term)?;
    let needs_load = {
        let state = state.borrow();
        family.is_truthy() && (state.loading == LoadState::Unloaded || state.name != family)
    };
    if needs_load {
        {
            let mut state = state.borrow_mut();
            state.font = None;
            state.loading = LoadState::Loading;
            state.name = family.clone();
        }
        load(term.clone(), family, Rc::clone(state), catalog.clone())?;
    }
    let ranges =
        native_ranges(&mut state.borrow_mut(), &from_js_string(text)).map_err(font_error)?;
    if let Some(ranges) = ranges {
        let result = Array::new();
        for [start, end] in ranges {
            // Source font ranges count glyph positions and fit the JS string's
            // u32 length. Do not remap supplementary text to UTF-16 offsets.
            result.push(&Array::of2(
                &JsValue::from_f64(start as f64),
                &JsValue::from_f64(end as f64),
            ));
        }
        return Ok(result);
    }
    fallback(text, patterns)
}
pub fn fallback(text: &JsString, patterns: &JsValue) -> Result<Array, JsValue> {
    let result = Array::new();
    let mut index = 0.0;
    while index < f64::from(text.length()) {
        let mut pattern_index = 0.0;
        let mut matched = false;
        while host::in_bounds(patterns, pattern_index)? {
            let pattern = host::pattern(patterns, pattern_index)?;
            if host::starts_with(text, &pattern, index)? {
                result.push(&Array::of2(
                    &JsValue::from_f64(index),
                    &host::end(patterns, pattern_index, index)?,
                ));
                let next = host::next(patterns, pattern_index, index)?;
                if next <= index {
                    return Err(js_sys::Error::new("Fallback ligature cannot advance").into());
                }
                index = next;
                matched = true;
                break;
            }
            let next = pattern_index + 1.0;
            if next == pattern_index {
                return Err(js_sys::Error::new("Fallback pattern index cannot advance").into());
            }
            pattern_index = next;
        }
        if !matched {
            index += 1.0;
        }
    }
    Ok(result)
}
