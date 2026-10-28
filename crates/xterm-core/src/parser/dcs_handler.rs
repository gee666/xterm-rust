// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of DcsHandler in common/parser/DcsParser.ts.

use super::{DcsHandlerTrait, DispatchError, HandlerResult, Params};
use crate::string_builder::LimitedStringBuilder;
use crate::text_decoder::utf32_to_string;
use std::cell::RefCell;
use std::rc::Rc;

/// ParserConstants.PAYLOAD_LIMIT, measured in JavaScript UTF-16 units.
pub const PAYLOAD_LIMIT: usize = 10_000_000;

struct Payload {
    data: LimitedStringBuilder,
    params: Params,
    hit_limit: bool,
}

fn empty_params() -> Params {
    let mut params = Params::default();
    params
        .reset_zdm()
        .expect("default Params has nonzero capacity");
    params
}

impl Payload {
    fn clear(&mut self) {
        self.params = empty_params();
        self.data.reset();
        self.hit_limit = false;
    }
}

type Callback = dyn FnMut(&[u16], &Params) -> HandlerResult;

/// Buffer a DCS payload and invoke one callback only after a successful unhook.
///
/// The callback receives UTF-16 units to preserve lone surrogates. Pending
/// futures must own any captured data. Cleanup occurs when they resolve, not
/// when dispatch pauses, matching upstream's retained payload lifetime.
pub struct DcsHandler {
    payload: Rc<RefCell<Payload>>,
    callback: Box<Callback>,
}

impl DcsHandler {
    pub fn new(callback: impl FnMut(&[u16], &Params) -> HandlerResult + 'static) -> Self {
        Self::with_limit(PAYLOAD_LIMIT, callback)
    }

    /// A bounded payload policy, also used to exercise limit boundaries cheaply.
    pub fn with_limit(
        limit: usize,
        callback: impl FnMut(&[u16], &Params) -> HandlerResult + 'static,
    ) -> Self {
        Self {
            payload: Rc::new(RefCell::new(Payload {
                data: LimitedStringBuilder::new(limit),
                params: empty_params(),
                hit_limit: false,
            })),
            callback: Box::new(callback),
        }
    }
}

impl DcsHandlerTrait for DcsHandler {
    fn hook(&mut self, params: &Params) -> Result<(), DispatchError> {
        let mut payload = self.payload.borrow_mut();
        payload.clear();
        // Upstream intentionally canonicalizes single zero or empty parameters,
        // including dropping subparameters on that zero.
        if params.len() > 1 || params.values().first().is_some_and(|&value| value != 0) {
            payload.params = params.clone();
        }
        Ok(())
    }

    fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        let mut payload = self.payload.borrow_mut();
        if !payload.hit_limit {
            payload.hit_limit = payload.data.append_utf16(&utf32_to_string(data));
        }
        Ok(())
    }

    fn unhook(&mut self, success: bool) -> HandlerResult {
        let result = {
            let payload = self.payload.borrow();
            if !payload.hit_limit && success {
                (self.callback)(payload.data.as_utf16(), &payload.params)
            } else {
                HandlerResult::Ready(false)
            }
        };
        match result {
            HandlerResult::Failed(error) => HandlerResult::Failed(error),
            HandlerResult::Ready(handled) => {
                self.payload.borrow_mut().clear();
                HandlerResult::Ready(handled)
            }
            HandlerResult::PendingFallible(future) => {
                let payload = Rc::clone(&self.payload);
                HandlerResult::PendingFallible(Box::pin(async move {
                    let handled = future.await?;
                    payload.borrow_mut().clear();
                    Ok(handled)
                }))
            }
            HandlerResult::Pending(future) => {
                let payload = Rc::clone(&self.payload);
                HandlerResult::Pending(Box::pin(async move {
                    let handled = future.await;
                    payload.borrow_mut().clear();
                    handled
                }))
            }
        }
    }
}
