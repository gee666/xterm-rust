// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Assertions ported from common/parser/Params.test.ts.
use xterm_core::parser::params::{ParamValue, Params, ParamsError};
use ParamValue::{Param as P, SubParams as S};

fn parse(params: &mut Params, chunks: &[&str]) {
    params.reset();
    params.add_param(0).unwrap();
    for chunk in chunks {
        for byte in chunk.bytes() {
            match byte {
                b';' => params.add_param(0).unwrap(),
                b':' => params.add_sub_param(-1).unwrap(),
                b'0'..=b'9' => params.add_digit(byte - b'0').unwrap(),
                _ => panic!("invalid test input"),
            }
        }
    }
}

fn from(values: &[ParamValue]) -> Params {
    Params::from_array(values).unwrap()
}

#[test]
fn respects_constructor_arguments() {
    let params = Params::with_capacities(12, 23).unwrap();
    assert_eq!(params.capacity(), 12);
    assert_eq!(params.sub_capacity(), 23);
    assert_eq!(params.to_array(), vec![]);
}

#[test]
fn add_param() {
    let mut params = Params::default();
    params.add_param(1).unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params.values(), [1]);
    assert_eq!(params.to_array(), vec![P(1)]);
    params.add_param(23).unwrap();
    assert_eq!(params.len(), 2);
    assert_eq!(params.values(), [1, 23]);
    assert_eq!(params.to_array(), vec![P(1), P(23)]);
    assert_eq!(params.sub_len(), 0);
}

#[test]
fn add_sub_param() {
    let mut params = Params::default();
    params.add_param(1).unwrap();
    params.add_sub_param(2).unwrap();
    params.add_sub_param(3).unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params.sub_len(), 2);
    assert_eq!(params.to_array(), vec![P(1), S(vec![2, 3])]);
    params.add_param(12345).unwrap();
    params.add_sub_param(-1).unwrap();
    assert_eq!(params.len(), 2);
    assert_eq!(params.sub_len(), 3);
    assert_eq!(
        params.to_array(),
        vec![P(1), S(vec![2, 3]), P(12345), S(vec![-1])]
    );
}

#[test]
fn sub_params_require_a_parameter() {
    let mut params = Params::default();
    params.add_sub_param(2).unwrap();
    params.add_sub_param(3).unwrap();
    assert_eq!(params.len(), 0);
    assert_eq!(params.sub_len(), 0);
    assert_eq!(params.to_array(), vec![]);
    params.add_param(1).unwrap();
    params.add_sub_param(2).unwrap();
    params.add_sub_param(3).unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params.sub_len(), 2);
    assert_eq!(params.to_array(), vec![P(1), S(vec![2, 3])]);
}

#[test]
fn reset() {
    let mut params = from(&[P(1), S(vec![2, 3]), P(12345), S(vec![-1])]);
    params.reset();
    assert_eq!(params.len(), 0);
    assert_eq!(params.sub_len(), 0);
    assert_eq!(params.to_array(), vec![]);
    params.add_param(1).unwrap();
    params.add_sub_param(2).unwrap();
    params.add_sub_param(3).unwrap();
    params.add_param(12345).unwrap();
    params.add_sub_param(-1).unwrap();
    assert_eq!(params.len(), 2);
    assert_eq!(params.sub_len(), 3);
    assert_eq!(
        params.to_array(),
        vec![P(1), S(vec![2, 3]), P(12345), S(vec![-1])]
    );
}

#[test]
fn array_roundtrip() {
    for data in [
        vec![],
        vec![P(1), S(vec![2, 3]), P(12345), S(vec![-1])],
        vec![P(38), P(2), P(50), P(100), P(150)],
        vec![P(38), P(2), P(50), P(100), S(vec![150])],
        vec![P(38), S(vec![2, 50, 100, 150])],
    ] {
        assert_eq!(from(&data).to_array(), data);
    }
    assert_eq!(
        from(&[P(38), S(vec![2, 50, 100, 150]), P(5), S(vec![]), P(6)]).to_array(),
        vec![P(38), S(vec![2, 50, 100, 150]), P(5), P(6)]
    );
}

#[test]
fn clone() {
    let params = from(&[
        P(38),
        S(vec![2, 50, 100, 150]),
        P(5),
        S(vec![]),
        P(6),
        P(1),
        S(vec![2, 3]),
        P(12345),
        S(vec![-1]),
    ]);
    assert_eq!(params.clone(), params);
}

#[test]
fn has_and_get_sub_params() {
    let params = from(&[P(38), S(vec![2, 50, 100, 150]), P(5), S(vec![]), P(6)]);
    assert!(params.has_sub_params(0));
    assert_eq!(params.get_sub_params(0), Some([2, 50, 100, 150].as_slice()));
    for index in [1, 2] {
        assert!(!params.has_sub_params(index));
        assert_eq!(params.get_sub_params(index), None);
    }
}

#[test]
fn get_all_sub_params() {
    let params = from(&[P(1), S(vec![2, 3]), P(7), P(12345), S(vec![-1])]);
    assert_eq!(
        params.get_sub_params_all(),
        [(0, vec![2, 3]), (2, vec![-1])].into()
    );
}

#[test]
fn zdm_default() {
    let mut params = Params::default();
    parse(&mut params, &[""]);
    assert_eq!(params.to_array(), vec![P(0)]);
}

#[test]
fn sub_default() {
    let mut params = Params::default();
    parse(&mut params, &[":"]);
    assert_eq!(params.to_array(), vec![P(0), S(vec![-1])]);
}

fn basic_sequences(params: &mut Params) {
    for (text, expected) in [
        ("1;2;3", vec![P(1), P(2), P(3)]),
        ("4", vec![P(4)]),
        ("4::123:5;6;7", vec![P(4), S(vec![-1, 123, 5]), P(6), P(7)]),
        ("", vec![P(0)]),
    ] {
        parse(params, &[text]);
        assert_eq!(params.to_array(), expected);
    }
}

#[test]
fn resets_on_new_sequence() {
    basic_sequences(&mut Params::default());
}

#[test]
fn length_restrictions() {
    let mut params = Params::with_capacities(3, 3).unwrap();
    basic_sequences(&mut params);
    parse(&mut params, &["1;2;3;4;5;6;7"]);
    assert_eq!(params.to_array(), vec![P(1), P(2), P(3)]);
    parse(&mut params, &["4;38:2::50:100:150;48:5:22"]);
    assert_eq!(
        params.to_array(),
        vec![P(4), P(38), S(vec![2, -1, 50]), P(48)]
    );
}

#[test]
fn typical_sequences() {
    let mut params = Params::default();
    for (text, expected) in [
        (
            "0;4;38;2;50;100;150;48;5;22",
            vec![
                P(0),
                P(4),
                P(38),
                P(2),
                P(50),
                P(100),
                P(150),
                P(48),
                P(5),
                P(22),
            ],
        ),
        (
            "0;4;38;2;50:100:150;48;5:22",
            vec![
                P(0),
                P(4),
                P(38),
                P(2),
                P(50),
                S(vec![100, 150]),
                P(48),
                P(5),
                S(vec![22]),
            ],
        ),
        (
            "0;4;38:2::50:100:150;48:5:22",
            vec![
                P(0),
                P(4),
                P(38),
                S(vec![2, -1, 50, 100, 150]),
                P(48),
                S(vec![5, 22]),
            ],
        ),
    ] {
        parse(&mut params, &[text]);
        assert_eq!(params.to_array(), expected);
    }
}

#[test]
fn reject_params_below_minus_one() {
    let mut params = Params::default();
    params.add_param(-1).unwrap();
    assert_eq!(params.add_param(-2), Err(ParamsError::ValueBelowDefault));
}

#[test]
fn reject_sub_params_below_minus_one() {
    let mut params = Params::default();
    params.add_param(-1).unwrap();
    params.add_sub_param(-1).unwrap();
    assert_eq!(
        params.add_sub_param(-2),
        Err(ParamsError::ValueBelowDefault)
    );
    assert_eq!(params.to_array(), vec![P(-1), S(vec![-1])]);
}

#[test]
fn clamp_parsed_params() {
    let mut params = Params::default();
    parse(&mut params, &["2147483648"]);
    assert_eq!(params.to_array(), vec![P(i32::MAX)]);
}

#[test]
fn clamp_parsed_sub_params() {
    let mut params = Params::default();
    parse(&mut params, &[":2147483648"]);
    assert_eq!(params.to_array(), vec![P(0), S(vec![i32::MAX])]);
}

#[test]
fn issue_2389_cancel_subdigits_beyond_params_limit() {
    let mut params = Params::default();
    parse(
        &mut params,
        &[";;;;;;;;;10;;;;;;;;;;20;;;;;;;;;;30;31;32;33;34;35::::::::"],
    );
    assert_eq!(
        params.values(),
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 30, 31, 32
        ]
    );
    assert_eq!(
        params.to_array(),
        params.values().iter().map(|v| P(*v)).collect::<Vec<_>>()
    );
}

#[test]
fn issue_2389_carry_forward_sub_state() {
    let mut params = Params::default();
    parse(&mut params, &["1:22:33", "44"]);
    assert_eq!(params.to_array(), vec![P(1), S(vec![22, 3344])]);
}

#[test]
fn additional_boundaries() {
    assert_eq!(
        Params::with_capacities(1, 257),
        Err(ParamsError::TooManySubParams)
    );
    let mut params = Params::default();
    assert!(params.is_empty());
    params.add_param(i64::MAX).unwrap();
    params.add_sub_param(i64::MAX).unwrap();
    assert_eq!(params.to_array(), vec![P(i32::MAX), S(vec![i32::MAX])]);
    params.reset_zdm().unwrap();
    assert_eq!(params.to_array(), vec![P(0)]);
    assert_eq!(params.add_digit(10), Err(ParamsError::InvalidDigit));
    assert_eq!(
        Params::with_capacities(0, 0).unwrap().reset_zdm(),
        Err(ParamsError::ZeroCapacityZdm)
    );
    assert_eq!(from(&[S(vec![99]), P(5)]).to_array(), vec![P(5)]);
    assert_eq!(params.get_sub_params(usize::MAX), None);
    let mut params = Params::with_capacities(2, 256).unwrap();
    params.add_param(1).unwrap();
    for _ in 0..256 {
        params.add_sub_param(2).unwrap();
    }
    // Match the upstream Uint16 packed offset overflow, not an idealized model.
    assert_eq!(params.sub_len(), 256);
    assert_eq!(params.to_array(), vec![P(1)]);
    params.add_param(3).unwrap();
    assert_eq!(params.to_array(), vec![P(1), P(3)]);
}
