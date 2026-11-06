//! Release native workload worker for scripts/matched-benchmark-adapter.mjs.
//! JSON decoding, construction, Unicode setup, and RICH2 capture are untimed.
use serde_json::{json, Value};
use std::{error::Error, fs, rc::Rc, time::Instant};
use xterm_core::{
    options::OptionValue,
    serialize::{SerializeAddon, SerializeOptions},
    terminal::CoreTerminal,
    unicode_addons::{Unicode11Addon, UnicodeGraphemesAddon},
};
#[path = "../../xterm-core/tests/support/terminal_snapshot.rs"]
mod base;
#[path = "../../xterm-core/tests/support/full_trace_snapshot.rs"]
mod rich;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

enum Operation {
    Text(Vec<u16>),
    Bytes(Vec<u8>),
    Resize(usize, usize),
    Serialize,
}
fn number(value: &Value) -> Result<usize> {
    Ok(usize::try_from(
        value.as_u64().ok_or("expected unsigned integer")?,
    )?)
}
fn operations(row: &Value) -> Result<Vec<Operation>> {
    row["operations"]
        .as_array()
        .ok_or("missing operations")?
        .iter()
        .map(|op| {
            Ok(match op["kind"].as_str().ok_or("missing operation kind")? {
                "write" => {
                    let text = op["units"].as_str().ok_or("missing prepared units")?;
                    if !text.is_empty() {
                        for unit in text.split(',') {
                            u16::from_str_radix(unit, 16)?;
                        }
                    }
                    // The validation above establishes the shared fixture decoder's invariant.
                    let units = base::parse_hex(text);
                    match op["encoding"].as_str() {
                        Some("utf16le") => Operation::Text(units),
                        Some("utf8-bytes") => Operation::Bytes(
                            units
                                .into_iter()
                                .map(u8::try_from)
                                .collect::<std::result::Result<_, _>>()?,
                        ),
                        _ => return Err("unsupported write encoding".into()),
                    }
                }
                "resize" => Operation::Resize(number(&op["cols"])?, number(&op["rows"])?),
                "addon"
                    if op["addon"] == "serialize"
                        && op["method"] == "serialize"
                        && op["arguments"] == json!([]) =>
                {
                    Operation::Serialize
                }
                _ => return Err("unsupported operation".into()),
            })
        })
        .collect()
}
fn sample(row: &Value, operations: &[Operation]) -> Result<Value> {
    let options = &row["options"];
    if options["settings"] != json!({"allowProposedApi": true})
        || options["unicode"]["settings"] != json!({})
    {
        return Err("unsupported options".into());
    }
    let mut terminal = CoreTerminal::new(
        number(&options["cols"])?,
        number(&options["rows"])?,
        number(&options["scrollback"])?,
    )?;
    terminal
        .state()
        .options
        .set("allowProposedApi", OptionValue::Bool(true))?;
    let unicode = Rc::clone(&terminal.state().unicode);
    let graphemes = UnicodeGraphemesAddon::new();
    match (
        options["unicode"]["provider"].as_str(),
        options["unicode"]["version"].as_str(),
    ) {
        (Some("builtin"), Some("6")) => {}
        (Some("unicode11"), Some("11")) => Unicode11Addon::new().activate(&unicode)?,
        (Some("unicode-graphemes"), Some("15-graphemes")) => {
            graphemes.activate(Rc::clone(&unicode))?
        }
        _ => return Err("unsupported Unicode configuration".into()),
    }
    unicode.set_active_version(
        options["unicode"]["version"]
            .as_str()
            .ok_or("missing Unicode version")?,
    )?;
    let serializer = SerializeAddon::new();
    let has_addon = operations
        .iter()
        .any(|op| matches!(op, Operation::Serialize));
    if has_addon {
        serializer.activate_engine(&terminal)?;
    }
    base::events(&mut terminal);
    let serialize_options = SerializeOptions::default();
    let mut outputs = Vec::new();
    let start = Instant::now();
    for operation in operations {
        match operation {
            Operation::Text(text) => terminal.write_utf16(text)?,
            Operation::Bytes(bytes) => terminal.write_bytes(bytes)?,
            Operation::Resize(cols, rows) => terminal.resize(*cols, *rows)?,
            Operation::Serialize => outputs.push(serializer.serialize(&serialize_options)?),
        }
    }
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    // Builtin handlers are synchronous. Never claim completion if a future or
    // parser-cleanup operation was produced by the executed workload.
    if terminal.take_pending().is_some() || !terminal.take_cleanup().is_empty() {
        return Err("unawaited native handler".into());
    }
    let trace = format!(
        "{}\t{}\n",
        rich::snapshot(&terminal)?,
        rich::events(&mut terminal)?
    );
    let addons = if has_addon {
        json!({"serialize": outputs})
    } else {
        json!({})
    };
    terminal.dispose()?;
    Ok(
        json!({"durationMs": duration_ms, "completion":"completed", "pendingOperations":0,
        "output":{"terminalTrace":trace,"addons":addons}}),
    )
}
fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    let capture_only = args.len() == 3 && args[0] == "--capture";
    if args.len() != 2 && !capture_only {
        return Err("usage: matched_benchmark [--capture] PREPARED.json OUTPUT.json, or --clock-probe OUTPUT.json".into());
    }
    if args[0] == "--clock-probe" {
        let mut previous = Instant::now();
        let mut minimum = None;
        for _ in 0..10_000 {
            let current = Instant::now();
            let nanos = current.duration_since(previous).as_nanos();
            if nanos > 0 {
                minimum = Some(minimum.map_or(nanos, |old: u128| old.min(nanos)));
            }
            previous = current;
        }
        let nanos = u64::try_from(minimum.ok_or("no positive monotonic clock delta")?)?;
        fs::write(
            &args[1],
            serde_json::to_vec(&json!({"minimumObservedDeltaNs":nanos,"reads":10000}))?,
        )?;
        return Ok(());
    }
    let input_path = if capture_only { &args[1] } else { &args[0] };
    let output_path = if capture_only { &args[2] } else { &args[1] };
    let input: Value = serde_json::from_slice(&fs::read(input_path)?)?;
    let warmup = number(&input["warmup"])?;
    let repetitions = number(&input["repetitions"])?;
    if warmup < 1 || repetitions < 10 {
        return Err("insufficient repetitions".into());
    }
    let mut cases = Vec::new();
    for row in input["cases"].as_array().ok_or("missing cases")? {
        let operations = operations(row)?;
        let run = |count| -> Result<Vec<Value>> {
            (0..count).map(|_| sample(row, &operations)).collect()
        };
        if capture_only {
            cases.push(json!({"id":row["id"],"output":sample(row, &operations)?["output"]}));
        } else {
            cases.push(
                json!({"id":row["id"],"warmup":run(warmup)?,"measurements":run(repetitions)?}),
            );
        }
    }
    fs::write(output_path, serde_json::to_vec(&cases)?)?;
    Ok(())
}
