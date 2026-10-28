// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.
use std::cell::Cell;
use std::collections::HashMap;
use std::io::{self, BufRead};
use std::rc::Rc;
use xterm_core::options::{
    CoercionError, CoercionHint, OptionObject, OptionValue as V, OptionsService,
};
#[path = "options_oracle/scenarios.rs"]
mod scenarios;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = OptionsService::default();
    let events = Rc::new(Cell::new(0_u64));
    let output = events.clone();
    service
        .on_option_change()
        .subscribe(move |_| output.set(output.get() + 1));
    let mut objects = HashMap::new();
    for line in io::stdin().lock().lines() {
        let line = line?;
        if line == "@defaults" {
            let defaults = OptionsService::default();
            for key in defaults.keys() {
                let value = defaults.get(key)?;
                let encoded = match &value {
                    V::Object(object) => {
                        if let Some(entries) =
                            object.payload::<std::rc::Rc<xterm_core::options::NativeProperties>>()
                        {
                            format!(
                                "o:{{{}}}",
                                entries
                                    .snapshot()
                                    .iter()
                                    .map(|(key, value)| format!("{key}={}", encode(value)))
                                    .collect::<Vec<_>>()
                                    .join(",")
                            )
                        } else {
                            "o:{}".into()
                        }
                    }
                    _ => encode(&value),
                };
                println!("{key}\t{encoded}");
            }
            continue;
        }
        if line == "@scenario" {
            println!("{}", scenarios::run()?);
            continue;
        }
        let constructor = line.starts_with("@new\t");
        let line = line.strip_prefix("@new\t").unwrap_or(&line);
        let fields: Vec<_> = line.split('\t').collect();
        let [key, kind, data] = fields.as_slice() else {
            return Err("expected key, kind, data".into());
        };
        let value = match *kind {
            "u" => V::Undefined,
            "z" => V::Null,
            "b" => V::Bool(*data == "true"),
            "n" => V::Number(data.parse()?),
            "s" => V::String(if data.is_empty() {
                Vec::new()
            } else {
                data.split(',')
                    .map(|unit| u16::from_str_radix(unit, 16))
                    .collect::<Result<_, _>>()?
            }),
            "o" => objects
                .entry(data.to_string())
                .or_insert_with(|| V::object(()))
                .clone(),
            "h" => {
                let name = data.to_string();
                objects
                    .entry(format!("hint:{data}"))
                    .or_insert_with(|| {
                        V::Object(OptionObject::new((), move |hint| match name.as_str() {
                            "split" => Ok(if hint == CoercionHint::Number {
                                V::Number(0.0)
                            } else {
                                V::from("zero")
                            }),
                            "bad" => Ok(V::object(())),
                            "throw" => Err(CoercionError::new(if hint == CoercionHint::Number {
                                "coercion number"
                            } else {
                                "coercion string"
                            })),
                            "numeric" => Ok(if hint == CoercionHint::Number {
                                V::Number(3.9)
                            } else {
                                V::from("three")
                            }),
                            _ => Err(CoercionError::new("unknown hint object")),
                        }))
                    })
                    .clone()
            }
            _ => return Err("unknown kind".into()),
        };
        if constructor {
            let constructed = OptionsService::new(false, [(*key, value)]);
            let diagnostics = constructed
                .diagnostics()
                .iter()
                .map(|e| encode_units(&e.message_utf16()))
                .collect::<Vec<_>>()
                .join(";");
            println!("new:{diagnostics}\t{}", encode(&constructed.get(key)?));
            continue;
        }
        let result = service.set(key, value);
        let prefix = match result {
            Ok(()) => "ok".into(),
            Err(error) => format!("error:{}", encode_units(&error.message_utf16())),
        };
        let actual = match service.get(key) {
            Ok(value) => encode(&value),
            Err(_) => "missing".into(),
        };
        println!("{prefix}\t{actual}\t{}", events.get());
    }
    Ok(())
}

fn encode_units(units: &[u16]) -> String {
    units
        .iter()
        .map(|unit| format!("{unit:x}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn encode(value: &V) -> String {
    match value {
        V::Undefined => "u".into(),
        V::Null => "z".into(),
        V::Bool(b) => format!("b:{b}"),
        V::Number(n) => format!("n:{}", ryu_js::Buffer::new().format(*n)),
        V::String(s) => format!(
            "s:{}",
            s.iter()
                .map(|v| format!("{v:x}"))
                .collect::<Vec<_>>()
                .join(",")
        ),
        V::Object(_) => "o".into(),
    }
}
