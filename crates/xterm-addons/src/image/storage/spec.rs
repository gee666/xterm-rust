// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{ImageResource, ImageSpec};

impl ImageSpec {
    pub fn actual_resource(&self) -> Option<ImageResource> {
        self.actual.borrow().clone()
    }
    /// Release the replaced surface after the RefCell guard has ended. A native
    /// surface destructor may call back into rendering or storage.
    pub fn replace_actual_resource(&self, resource: Option<ImageResource>) {
        let previous = self.actual.replace(resource);
        drop(previous);
    }
}
