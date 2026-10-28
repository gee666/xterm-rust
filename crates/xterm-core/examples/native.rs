// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
//! Interactive, line-oriented demonstration of the real Rust terminal library.
//! This is not a PTY shell or proof of complete xterm.js equivalence.
#[path = "native/arguments.rs"]
mod arguments;
#[path = "native/commands.rs"]
mod commands;
#[path = "native/escape.rs"]
mod escape;
#[path = "native/view.rs"]
mod view;

use commands::{Demo, DemoResult, Outcome};
use std::{
    fs::File,
    io::{self, BufRead, BufReader, IsTerminal, Write},
    path::Path,
};

const HELP: &str = "Commands: sample | write TEXT | input TEXT | resize COLS ROWS | show | serialize | reset | help | quit
Escapes in TEXT: \\e \\n \\r \\t \\\\ \\xHH \\uHHHH. Input emits outbound data, not display echo.
Script checks: assert-size COLS ROWS | assert-cursor X Y | assert-line ROW TEXT
assert-cell COL ROW TEXT | assert-fg COL ROW PALETTE | assert-width COL ROW WIDTH | assert-output TEXT
This example executes the Rust parser and buffers. Full xterm.js equivalence is NOT PROVEN.";

fn main() -> DemoResult<()> {
    let args = arguments::parse(std::env::args_os().skip(1))?;
    if args.help {
        writeln!(io::stdout().lock(), "cargo run -p xterm-core --example native -- [--plain] [--script FILE] [--max-cells N]\n{HELP}")?;
        return Ok(());
    }
    let mut demo = match args.cell_budget {
        Some(budget) => Demo::with_cell_budget(budget)?,
        None => Demo::new()?,
    };
    let run = match args.script.as_deref() {
        Some(path) => run_script(&mut demo, path, args.plain),
        None => run_interactive(&mut demo, args.plain),
    };
    let cleanup = demo.close();
    if let Err(error) = run {
        return match cleanup {
            Ok(()) => Err(error),
            Err(cleanup) => Err(format!("{error}; cleanup also failed: {cleanup}").into()),
        };
    }
    cleanup?;
    writeln!(
        io::stdout().lock(),
        "Assertions: {}; disposed: {}",
        demo.assertions(),
        demo.terminal.is_disposed()?
    )?;
    Ok(())
}

fn present(demo: &mut Demo, line: &str, plain: bool) -> DemoResult<bool> {
    match demo.execute(line)? {
        Outcome::Frame => write!(io::stdout().lock(), "{}", view::frame(demo, plain)?)?,
        Outcome::Message(message) => writeln!(io::stdout().lock(), "{message}")?,
        Outcome::Help => writeln!(io::stdout().lock(), "{HELP}")?,
        Outcome::Quit => return Ok(false),
        Outcome::Empty => {}
    }
    Ok(true)
}

fn run_script(demo: &mut Demo, path: &Path, plain: bool) -> DemoResult<()> {
    let input = BufReader::new(File::open(path).map_err(|error| format!("{path:?}: {error}"))?);
    for (index, line) in input.lines().enumerate() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if !present(demo, &line, plain)
            .map_err(|error| format!("{path:?}:{}: {error}", index + 1))?
        {
            break;
        }
    }
    Ok(())
}

fn run_interactive(demo: &mut Demo, plain: bool) -> DemoResult<()> {
    writeln!(io::stdout().lock(), "{HELP}")?;
    present(demo, "sample", plain)?;
    let interactive = io::stdin().is_terminal();
    let input = io::stdin();
    let mut input = input.lock();
    loop {
        if interactive {
            write!(io::stdout().lock(), "xterm-rust> ")?;
            io::stdout().flush()?;
        }
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim_end_matches(['\r', '\n']);
        match present(demo, line, plain) {
            Ok(true) => {}
            Ok(false) => break,
            Err(error) => writeln!(io::stderr().lock(), "error: {error}")?,
        }
    }
    Ok(())
}
