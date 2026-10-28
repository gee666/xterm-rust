// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::{AttributeData, Buffer, LineRef},
    lifecycle::DisposableHandle,
    options::{OptionValue as V, OptionsService},
};

pub struct Harness {
    pub buffer: Rc<RefCell<Buffer>>,
    pub options: Rc<OptionsService>,
    identities: RefCell<Vec<LineRef>>,
    out: RefCell<Vec<String>>,
    subscriptions: RefCell<Vec<DisposableHandle>>,
}
impl Harness {
    pub fn new(cols: usize, rows: usize) -> Rc<Self> {
        let options = Rc::new(OptionsService::default());
        options.set_raw("scrollback", V::Number(3.0)).unwrap();
        let mut buffer = Buffer::new(true, cols, rows, options.clone()).unwrap();
        buffer.fill_viewport_rows(None).unwrap();
        let identities = (0..buffer.lines.len())
            .map(|i| buffer.lines.get(i).unwrap())
            .collect();
        Rc::new(Self {
            buffer: Rc::new(RefCell::new(buffer)),
            options,
            identities: RefCell::new(identities),
            out: RefCell::new(Vec::new()),
            subscriptions: RefCell::new(Vec::new()),
        })
    }
    pub fn snapshot(&self, tag: &str) {
        let b = self.buffer.borrow();
        let meta = [
            b.cols().to_string(),
            b.rows().to_string(),
            b.x.to_string(),
            b.y.to_string(),
            b.ybase.to_string(),
            b.ydisp.to_string(),
            b.saved_x.to_string(),
            b.saved_y.to_string(),
            b.scroll_top.to_string(),
            b.scroll_bottom.to_string(),
            b.lines.len().to_string(),
            b.lines.max_length().to_string(),
        ]
        .join(",");
        let mut rows = Vec::new();
        for y in 0..b.lines.len() {
            let Some(line) = b.lines.get(y) else {
                rows.push("_".to_owned());
                continue;
            };
            let identity = {
                let mut ids = self.identities.borrow_mut();
                if let Some(i) = ids
                    .iter()
                    .position(|candidate| Rc::ptr_eq(candidate, &line))
                {
                    i + 1
                } else {
                    ids.push(line.clone());
                    ids.len()
                }
            };
            let line = line.borrow();
            let cells = (0..line.len())
                .map(|x| {
                    let c = line.get_cell(x).unwrap();
                    let chars = c.get_chars();
                    let text = if chars.is_empty() {
                        "-".to_owned()
                    } else {
                        chars
                            .iter()
                            .map(|unit| format!("{unit:x}"))
                            .collect::<Vec<_>>()
                            .join(",")
                    };
                    format!(
                        "{}:{}:{}:{}:{}:{}",
                        c.content,
                        c.fg,
                        c.bg,
                        c.extended.ext(),
                        c.extended.url_id(),
                        text
                    )
                })
                .collect::<Vec<_>>()
                .join("/");
            rows.push(format!(
                "{};{};{}",
                identity,
                usize::from(line.is_wrapped),
                cells
            ));
        }
        self.out
            .borrow_mut()
            .push(format!("{tag}#{meta}#{}", rows.join("|")));
    }
    pub fn resize(&self, cols: usize, rows: usize) {
        Buffer::resize_shared(&self.buffer, cols, rows).unwrap();
    }
    pub fn letters(&self, row: usize, text: &str) {
        let line = self.buffer.borrow().lines.get(row).unwrap();
        for (x, ch) in text.chars().enumerate() {
            line.borrow_mut().set_cell_from_codepoint(
                x,
                u32::from(ch),
                1,
                &AttributeData::default(),
            );
        }
    }
    pub fn retain(&self, subscription: DisposableHandle) {
        self.subscriptions.borrow_mut().push(subscription);
    }
    pub fn output(&self) -> String {
        self.out.borrow().join("^")
    }
    pub fn assert_fixture(&self, name: &str) {
        let fixture = include_str!("../fixtures/buffer-resize-reentry.tsv");
        assert_eq!(fixture.lines().count(), 10);
        let expected = fixture
            .lines()
            .find_map(|line| {
                line.split_once('\t')
                    .filter(|(label, _)| *label == name)
                    .map(|(_, trace)| trace)
            })
            .unwrap();
        assert_eq!(self.output(), expected, "case {name}");
    }
}
impl Drop for Harness {
    fn drop(&mut self) {
        for subscription in self.subscriptions.get_mut().drain(..) {
            subscription.dispose();
        }
    }
}
pub fn trimmed() -> Rc<Harness> {
    let h = Harness::new(4, 3);
    {
        let mut b = h.buffer.borrow_mut();
        for _ in 0..3 {
            b.lines
                .push(b.get_blank_line(&AttributeData::default(), false))
                .unwrap();
        }
        b.y = 2;
        b.ybase = 3;
        b.ydisp = 3;
    }
    h.options.set_raw("scrollback", V::Number(0.0)).unwrap();
    h
}
pub fn larger() -> Rc<Harness> {
    let h = Harness::new(2, 5);
    h.letters(0, "ab");
    h.letters(1, "cd");
    h.buffer.borrow().set_wrapped(1, true).unwrap();
    h.buffer.borrow_mut().y = 4;
    h
}
pub fn smaller() -> Rc<Harness> {
    let h = Harness::new(4, 5);
    h.letters(0, "abcd");
    h.buffer.borrow_mut().y = 4;
    h
}
#[derive(Debug)]
pub struct ObserverFailure;
impl std::fmt::Display for ObserverFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("resize observer failed")
    }
}
impl std::error::Error for ObserverFailure {}
