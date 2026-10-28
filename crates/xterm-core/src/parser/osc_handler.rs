// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of common/parser/OscParser.ts.

use super::{HandlerResult, OscError, OscHandlerTrait, PAYLOAD_LIMIT};
use crate::string_builder::LimitedStringBuilder;
use crate::text_decoder::utf32_to_string;
use std::cell::RefCell;
use std::rc::Rc;

struct Payload {
    data: LimitedStringBuilder,
    hit_limit: bool,
}

/// Adapts a whole UTF-16 payload callback to the streaming handler contract.
///
/// The default limit is upstream's ten million UTF-16 code units. A custom limit
/// supports protocol adapters and tests without process-global mutable settings.
/// The callback receives owned data so its future may retain it while pending.
/// Successful future completion clears shared accumulation like upstream's then
/// callback. A dropped future leaves it intact until the next start.
pub struct OscHandler<F> {
    payload: Rc<RefCell<Payload>>,
    callback: F,
}

impl<F: FnMut(Vec<u16>) -> HandlerResult> OscHandler<F> {
    pub fn new(callback: F) -> Self {
        Self::with_limit(callback, PAYLOAD_LIMIT)
    }

    pub fn with_limit(callback: F, limit: usize) -> Self {
        Self {
            payload: Rc::new(RefCell::new(Payload {
                data: LimitedStringBuilder::new(limit),
                hit_limit: false,
            })),
            callback,
        }
    }
}

fn clear(payload: &RefCell<Payload>) {
    let mut payload = payload.borrow_mut();
    payload.data.reset();
    payload.hit_limit = false;
}

impl<F: FnMut(Vec<u16>) -> HandlerResult> OscHandlerTrait for OscHandler<F> {
    fn start(&mut self) -> Result<(), OscError> {
        clear(&self.payload);
        Ok(())
    }

    fn put(&mut self, data: &[u32]) -> Result<(), OscError> {
        let mut payload = self.payload.borrow_mut();
        if !payload.hit_limit && payload.data.append_utf16(&utf32_to_string(data)) {
            payload.hit_limit = true;
        }
        Ok(())
    }

    fn end(&mut self, success: bool) -> HandlerResult {
        let data = {
            let payload = self.payload.borrow();
            if success && !payload.hit_limit {
                Some(payload.data.as_utf16().to_vec())
            } else {
                None
            }
        };
        let result = data.map_or(HandlerResult::Ready(false), &mut self.callback);
        match result {
            // A source callback throwing bypasses the following cleanup. Keep
            // accumulation intact until the next start, rather than swallowing it.
            HandlerResult::Failed(error) => HandlerResult::Failed(error),
            HandlerResult::Ready(value) => {
                clear(&self.payload);
                HandlerResult::Ready(value)
            }
            HandlerResult::PendingFallible(future) => {
                let payload = Rc::clone(&self.payload);
                HandlerResult::PendingFallible(Box::pin(async move {
                    let result = future.await?;
                    clear(&payload);
                    Ok(result)
                }))
            }
            HandlerResult::Pending(future) => {
                let payload = Rc::clone(&self.payload);
                HandlerResult::Pending(Box::pin(async move {
                    let result = future.await;
                    clear(&payload);
                    result
                }))
            }
        }
    }
}
