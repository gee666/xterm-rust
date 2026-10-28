// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use xterm_core::parser::apc::ApcHandlerTrait;

/// Actual portable parser adapter. It forwards fallible start/put immediately,
/// while owned display futures remain valid after addon reset or disposal.
pub struct KittyApcAdapter {
    controller: Rc<KittyGraphicsHandler>,
}
impl KittyApcAdapter {
    pub fn new(controller: Rc<KittyGraphicsHandler>) -> Self {
        Self { controller }
    }
}
impl ApcHandlerTrait for KittyApcAdapter {
    fn start(&mut self) -> Result<(), DispatchError> {
        self.controller.start().map_err(dispatch_error)
    }
    fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        self.controller.put(data).map_err(dispatch_error)
    }
    fn end(&mut self, success: bool) -> HandlerResult {
        match self.controller.end(success) {
            Ok(result) => result,
            Err(error) => HandlerResult::Failed(dispatch_error(error)),
        }
    }
}
