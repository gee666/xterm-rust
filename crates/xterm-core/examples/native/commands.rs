// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
//! Commands are shared by the interactive loop and deterministic script mode.
#[path = "checks.rs"]
mod checks;
use super::{diagnostics, escape, view};
use std::{cell::RefCell, error::Error, rc::Rc};
use xterm_core::{headless::Terminal, lifecycle::DisposableHandle};

pub type DemoResult<T> = Result<T, Box<dyn Error>>;
const INITIAL_COLS: usize = 40;
const INITIAL_ROWS: usize = 8;
const SCROLLBACK: usize = 100;
pub const DEFAULT_CELL_BUDGET: usize = 1_000_000;
const SAMPLE: &str =
    "\x1b[31mred\x1b[0m \x1b[32mgreen\x1b[0m\r\nA😀界e\u{301}\r\ncursor: abc\x1b[3DXYZ";

pub enum Outcome {
    Frame,
    Message(String),
    Help,
    Quit,
    Empty,
}

pub struct Demo {
    pub terminal: Terminal,
    outbound: Rc<RefCell<Vec<u16>>>,
    subscription: DisposableHandle,
    assertions: usize,
    cell_budget: usize,
}

impl Demo {
    pub fn new() -> DemoResult<Self> {
        Self::with_cell_budget(DEFAULT_CELL_BUDGET)
    }

    pub fn with_cell_budget(cell_budget: usize) -> DemoResult<Self> {
        if cell_budget < INITIAL_COLS * INITIAL_ROWS {
            return Err("cell budget is smaller than initial viewport".into());
        }
        let terminal = Terminal::new(INITIAL_COLS, INITIAL_ROWS, SCROLLBACK)?;
        if let Err(error) = diagnostics::install(&terminal) {
            return match terminal.dispose() {
                Ok(()) => Err(error),
                Err(cleanup) => Err(format!("{error}; cleanup also failed: {cleanup}").into()),
            };
        }
        let outbound = Rc::new(RefCell::new(Vec::new()));
        let capture = Rc::clone(&outbound);
        let subscription = terminal.on_data().subscribe(move |data| {
            capture.borrow_mut().extend_from_slice(data);
        });
        Ok(Self {
            terminal,
            outbound,
            subscription,
            assertions: 0,
            cell_budget,
        })
    }

    pub fn assertions(&self) -> usize {
        self.assertions
    }

    pub fn outbound(&self) -> Vec<u16> {
        self.outbound.borrow().clone()
    }

    pub fn close(&self) -> DemoResult<()> {
        // Explicit cleanup is fallible. Do not hide a terminal callback failure in Drop.
        let result = self.terminal.dispose();
        self.subscription.dispose();
        result?;
        Ok(())
    }

    pub fn execute(&mut self, line: &str) -> DemoResult<Outcome> {
        let (command, argument) = line.split_once(' ').unwrap_or((line, ""));
        if command.starts_with("assert-") {
            self.check(command, argument)?;
            self.assertions += 1;
            return Ok(Outcome::Message(format!("ok {command}")));
        }
        match command {
            "" => Ok(Outcome::Empty),
            "write" => {
                self.terminal.write_utf16(&escape::decode(argument)?)?;
                Ok(Outcome::Frame)
            }
            "input" => {
                self.terminal.input(&escape::decode(argument)?, true)?;
                Ok(Outcome::Frame)
            }
            "resize" => {
                let values = numbers(argument, 2)?;
                if values[0] < 2 || values[1] == 0 {
                    return Err("resize requires at least 2 columns and 1 row".into());
                }
                let cells = values[0]
                    .checked_mul(values[1])
                    .ok_or("viewport cell count overflow")?;
                if cells > self.cell_budget {
                    return Err("viewport exceeds --max-cells budget".into());
                }
                self.terminal.resize(values[0], values[1])?;
                Ok(Outcome::Frame)
            }
            "sample" | "reset" | "show" | "serialize" | "quit" | "help" => {
                if !argument.is_empty() {
                    return Err(format!("{command} takes no arguments").into());
                }
                match command {
                    "sample" => {
                        self.terminal.reset()?;
                        self.terminal.write(SAMPLE)?;
                        Ok(Outcome::Frame)
                    }
                    "reset" => {
                        self.terminal.reset()?;
                        Ok(Outcome::Frame)
                    }
                    "show" => Ok(Outcome::Frame),
                    "serialize" => Ok(Outcome::Message(view::serialized(&self.terminal)?)),
                    "quit" => Ok(Outcome::Quit),
                    _ => Ok(Outcome::Help),
                }
            }
            _ => Err(format!("unknown command {command:?}; use help").into()),
        }
    }
}

pub(super) fn numbers(text: &str, count: usize) -> DemoResult<Vec<usize>> {
    // One extra token is enough to reject wrong arity without collecting an
    // arbitrarily long malformed command into a temporary vector.
    let limit = count.checked_add(1).ok_or("argument count overflow")?;
    let values = text
        .split_whitespace()
        .take(limit)
        .map(str::parse)
        .collect::<Result<Vec<_>, _>>()?;
    if values.len() != count {
        return Err(format!("expected {count} unsigned integers").into());
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sample_checks_real_cells_colors_unicode_and_cursor_overwrite() {
        let mut demo = Demo::new().unwrap();
        demo.execute("sample").unwrap();
        for command in [
            "assert-line 0 red green",
            "assert-fg 0 0 1",
            "assert-fg 4 0 2",
            "assert-cell 1 1 😀",
            // The source defaults to Unicode 6: this emoji is narrow, CJK is wide.
            "assert-width 1 1 1",
            "assert-width 2 1 2",
            "assert-cell 4 1 e\\u0301",
            "assert-line 2 cursor: XYZ",
            "assert-cursor 11 2",
        ] {
            demo.execute(command).unwrap();
        }
        assert!(view::frame(&demo, false).unwrap().contains("\x1b[38;5;1m"));
        assert!(!view::frame(&demo, true).unwrap().contains('\x1b'));
        assert!(!view::serialized(&demo.terminal).unwrap().is_empty());
        demo.close().unwrap();
        assert!(demo.terminal.is_disposed().unwrap());
    }
    #[test]
    fn input_is_outbound_not_echo_and_bad_commands_are_atomic() {
        let mut demo = Demo::new().unwrap();
        demo.execute("input hi\\r\\uD800").unwrap();
        demo.execute("assert-output hi\\r\\uD800").unwrap();
        demo.execute("assert-line 0 ").unwrap();
        for command in [
            "resize 0 2",
            "resize 2 0",
            "resize -1 3",
            "resize 1000000 1000000",
            "resize 4",
            "reset x",
            "write \\xGG",
            "input partial\\xGG",
            "bogus",
        ] {
            assert!(demo.execute(command).is_err(), "{command}");
        }
        demo.execute("assert-size 40 8").unwrap();
        demo.execute("assert-cursor 0 0").unwrap();
        demo.execute("assert-output hi\\r\\uD800").unwrap();
        assert!(demo.execute("assert-cell 100 0 x").is_err());
        assert!(demo.execute("assert-line 100 x").is_err());
        assert!(demo.execute("assert-size 1 1").is_err());
        demo.close().unwrap();
    }
    #[test]
    fn cursor_wrap_group_keeps_source_default_reflow_exclusion() {
        let mut demo = Demo::new().unwrap();
        for command in [
            "resize 8 4",
            "write abcdefghijkl",
            "resize 12 4",
            "assert-line 0 abcdefgh",
            "assert-line 1 ijkl",
            "assert-cursor 4 1",
        ] {
            demo.execute(command).unwrap();
        }
        demo.close().unwrap();
    }

    #[test]
    fn resize_reflows_actual_rows_and_script_observes_state() {
        let mut demo = Demo::new().unwrap();
        for command in [
            "resize 8 4",
            // Default reflow excludes the cursor's wrapped line, like upstream.
            // Move below that group before widening so real reflow participates.
            "write abcdefghijkl\\r\\n",
            "assert-line 0 abcdefgh",
            "assert-line 1 ijkl",
            "resize 12 4",
            "assert-size 12 4",
            "assert-line 0 abcdefghijkl",
            "assert-cursor 0 1",
        ] {
            demo.execute(command).unwrap();
        }
        assert!(matches!(demo.execute("quit").unwrap(), Outcome::Quit));
        assert!(matches!(demo.execute("help").unwrap(), Outcome::Help));
        assert!(matches!(demo.execute("").unwrap(), Outcome::Empty));
        demo.close().unwrap();
    }
}
