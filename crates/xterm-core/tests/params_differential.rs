// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use std::fmt::Write;
use xterm_core::parser::params::{ParamValue, Params};

fn snapshot(params: &Params, failed: bool) -> String {
    let mut result = String::from(if failed { "![" } else { "[" });
    for (index, value) in params.to_array().iter().enumerate() {
        if index > 0 {
            result.push(',');
        }
        match value {
            ParamValue::Param(value) => write!(result, "{value}").unwrap(),
            ParamValue::SubParams(values) => {
                result.push('[');
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        result.push(',');
                    }
                    write!(result, "{value}").unwrap();
                }
                result.push(']');
            }
        }
    }
    result.push(']');
    result
}

#[test]
fn seeded_upstream_operation_traces() {
    let corpus = include_str!("fixtures/params-oracle.tsv");
    let mut count = 0;
    for (case, line) in corpus.lines().enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 4);
        let mut params =
            Params::with_capacities(fields[0].parse().unwrap(), fields[1].parse().unwrap())
                .unwrap();
        let operations: Vec<_> = fields[2].split(',').collect();
        let expected: Vec<_> = fields[3].split('|').collect();
        assert_eq!(operations.len(), expected.len());
        for (step, (op, expected)) in operations.iter().zip(expected).enumerate() {
            let result = match &op[..1] {
                "p" => params.add_param(op[1..].parse().unwrap()),
                "s" => params.add_sub_param(op[1..].parse().unwrap()),
                "d" => params.add_digit(op[1..].parse().unwrap()),
                "r" => {
                    params.reset();
                    Ok(())
                }
                "z" => params.reset_zdm(),
                "c" => {
                    params = params.clone();
                    Ok(())
                }
                _ => panic!("invalid corpus operation"),
            };
            assert_eq!(
                snapshot(&params, result.is_err()),
                expected,
                "seed 0x2389abcd case {case} step {step} op {op}"
            );
            for index in 0..params.len() {
                let all = params.get_sub_params_all();
                assert_eq!(params.has_sub_params(index), all.contains_key(&index));
                assert_eq!(
                    params.get_sub_params(index),
                    all.get(&index).map(Vec::as_slice)
                );
            }
            count += 1;
        }
    }
    assert_eq!(count, 49_409);
}
