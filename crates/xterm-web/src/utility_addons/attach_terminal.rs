//! A real queued browser writer with shared native event subscriptions.
use std::rc::Rc;
use wasm_bindgen::JsValue;
use xterm_addons::{attach::*, native_utility::NativeUtilityTerminal};
use xterm_core::{input::write_buffer::WriteData, lifecycle::DisposableHandle};

pub type QueuedWriter = Rc<dyn Fn(WriteData) -> Result<(), JsValue>>;
pub struct BrowserAttachTerminal {
    pub native: Rc<NativeUtilityTerminal>,
    pub writer: QueuedWriter,
}
impl AttachTerminal for BrowserAttachTerminal {
    fn write(&self, data: SocketMessage) -> Result<(), AttachError> {
        let data = match data {
            SocketMessage::Text(data) => WriteData::Text(data),
            SocketMessage::Binary(data) => WriteData::Bytes(data),
        };
        (self.writer)(data)
            .map_err(|error| AttachError::host(crate::js_error::JsCallbackError(error)))
    }
    fn on_data(&self, callback: DataCallback) -> DisposableHandle {
        self.native.on_data(callback)
    }
    fn on_binary(&self, callback: DataCallback) -> DisposableHandle {
        self.native.on_binary(callback)
    }
}
