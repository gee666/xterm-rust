// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use xterm_core::parser::params::{ParamValue as V, Params, ParamsError};

#[test]
fn default_and_zero_capacity_contracts() {
    let defaults = Params::default();
    assert_eq!((defaults.capacity(), defaults.sub_capacity()), (32, 32));
    assert_eq!((defaults.len(), defaults.sub_len()), (0, 0));
    assert!(defaults.is_empty());
    assert!(defaults.values().is_empty());
    let mut zero = Params::with_capacities(0, 0).unwrap();
    // Upstream rejects capacity before validating the numeric argument.
    zero.add_param(-2).unwrap();
    zero.add_sub_param(-2).unwrap();
    zero.add_digit(9).unwrap();
    assert!(zero.to_array().is_empty());
    let before = zero.clone();
    assert_eq!(zero.reset_zdm(), Err(ParamsError::ZeroCapacityZdm));
    assert_eq!(zero, before);
}

#[test]
fn owned_subparameter_snapshots_do_not_alias_mutations() {
    let mut params = Params::from_array(&[V::Param(1), V::SubParams(vec![2, 3])]).unwrap();
    let snapshot = params.get_sub_params_all();
    let array = params.to_array();
    params.add_digit(4).unwrap();
    assert_eq!(params.get_sub_params(0), Some([2, 34].as_slice()));
    assert_eq!(snapshot[&0], [2, 3]);
    assert_eq!(array, vec![V::Param(1), V::SubParams(vec![2, 3])]);
    params.reset();
    assert!(!params.has_sub_params(0));
    assert_eq!(params.get_sub_params(0), None);
    assert!(params.get_sub_params_all().is_empty());
    assert!(params.values().is_empty());
}

#[test]
fn array_conversion_keeps_source_capacity_and_leading_array_rules() {
    let params = Params::from_array(&[
        V::SubParams(vec![9]),
        V::SubParams(vec![8]),
        V::Param(7),
        V::SubParams(vec![]),
        V::Param(6),
    ])
    .unwrap();
    assert_eq!(params.to_array(), vec![V::Param(7), V::Param(6)]);
    let overlong: Vec<_> = (0..33).map(V::Param).collect();
    let params = Params::from_array(&overlong).unwrap();
    assert_eq!(params.len(), 32);
    assert_eq!(params.values(), (0..32).collect::<Vec<_>>());
    let params = Params::from_array(&[V::Param(1), V::SubParams((0..33).collect())]).unwrap();
    assert_eq!(params.sub_len(), 32);
    assert_eq!(
        params.get_sub_params(0).unwrap(),
        (0..32).collect::<Vec<_>>()
    );
}

#[test]
fn rejection_precedence_and_target_switching() {
    let mut params = Params::with_capacities(1, 1).unwrap();
    params.add_sub_param(-2).unwrap();
    params.add_param(-1).unwrap();
    params.add_digit(8).unwrap();
    params.add_sub_param(-1).unwrap();
    params.add_digit(9).unwrap();
    params.add_sub_param(-2).unwrap();
    params.add_digit(7).unwrap();
    assert_eq!(params.to_array(), vec![V::Param(8), V::SubParams(vec![9])]);
    params.add_param(-2).unwrap();
    params.add_digit(6).unwrap();
    params.add_sub_param(5).unwrap();
    assert_eq!(params.to_array(), vec![V::Param(8), V::SubParams(vec![9])]);
    // Both rejection flags must be copied, not just logical values.
    let mut cloned = params.clone();
    cloned.add_digit(4).unwrap();
    assert_eq!(cloned, params);
    cloned.reset_zdm().unwrap();
    cloned.add_digit(2).unwrap();
    cloned.add_sub_param(-1).unwrap();
    cloned.add_digit(3).unwrap();
    assert_eq!(cloned.to_array(), vec![V::Param(2), V::SubParams(vec![3])]);
    assert_eq!(params.to_array(), vec![V::Param(8), V::SubParams(vec![9])]);
}

#[test]
fn clone_continues_sub_digits_without_affecting_original() {
    let mut params = Params::from_array(&[V::Param(1), V::SubParams(vec![22])]).unwrap();
    let mut cloned = params.clone();
    cloned.add_digit(3).unwrap();
    assert_eq!(
        cloned.to_array(),
        vec![V::Param(1), V::SubParams(vec![223])]
    );
    assert_eq!(params.to_array(), vec![V::Param(1), V::SubParams(vec![22])]);
    params.reset();
    params.add_digit(9).unwrap();
    assert!(params.is_empty());
    params.add_param(4).unwrap();
    params.add_digit(5).unwrap();
    assert_eq!(params.to_array(), vec![V::Param(45)]);
}

#[test]
fn thrown_validation_still_switches_digit_target_as_upstream() {
    let mut params = Params::from_array(&[V::Param(1), V::SubParams(vec![2])]).unwrap();
    assert_eq!(params.add_param(-2), Err(ParamsError::ValueBelowDefault));
    params.add_digit(3).unwrap();
    assert_eq!(params.to_array(), vec![V::Param(13), V::SubParams(vec![2])]);
    assert_eq!(
        params.add_sub_param(-2),
        Err(ParamsError::ValueBelowDefault)
    );
    params.add_digit(4).unwrap();
    assert_eq!(
        params.to_array(),
        vec![V::Param(13), V::SubParams(vec![24])]
    );
}

#[test]
fn exact_direct_and_decimal_saturation_boundaries() {
    let mut params = Params::default();
    for value in [i64::from(i32::MAX), i64::from(i32::MAX) + 1, i64::MAX] {
        params.add_param(value).unwrap();
        params.add_sub_param(value).unwrap();
    }
    assert_eq!(params.values(), [i32::MAX; 3]);
    for index in 0..3 {
        assert_eq!(params.get_sub_params(index), Some([i32::MAX].as_slice()));
    }
    params.reset_zdm().unwrap();
    for byte in b"2147483647" {
        params.add_digit(byte - b'0').unwrap();
    }
    assert_eq!(params.values(), [i32::MAX]);
    for _ in 0..100 {
        params.add_digit(9).unwrap();
    }
    assert_eq!(params.values(), [i32::MAX]);
}
