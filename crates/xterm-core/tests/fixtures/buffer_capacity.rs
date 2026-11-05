// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// One assertion set is shared by native and wasm32 test harnesses.
use std::rc::Rc;
use xterm_core::{
    buffer::Buffer,
    options::{OptionValue, OptionsService},
};

const SOURCE_MAX: usize = u32::MAX as usize;

pub fn verify_capacity_boundaries() {
    for (scrollback_enabled, rows, scrollback, expected) in [
        (false, 1, 0, 1),
        (false, 4, SOURCE_MAX, 4),
        (true, 24, 1000, 1024),
        (true, SOURCE_MAX - 1, 1, SOURCE_MAX),
        (true, SOURCE_MAX, 1, SOURCE_MAX),
        (true, 1, SOURCE_MAX, SOURCE_MAX),
        (true, SOURCE_MAX, SOURCE_MAX, SOURCE_MAX),
    ] {
        let options = Rc::new(OptionsService::default());
        options
            .set("scrollback", OptionValue::Number(scrollback as f64))
            .unwrap();
        let buffer = Buffer::new(scrollback_enabled, 1, rows, options).unwrap();
        assert_eq!(
            buffer.lines.max_length(),
            expected,
            "rows={rows}, scrollback={scrollback}"
        );
        // Source-supported huge capacities must remain sparse, not allocate a
        // viewport merely to validate arithmetic at the protocol limit.
        assert_eq!(buffer.lines.len(), 0);
    }
}
