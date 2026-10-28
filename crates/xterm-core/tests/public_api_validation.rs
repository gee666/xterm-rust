use xterm_core::public_api::validation::{
    verify_integers, verify_mutable_option, verify_positive_integers, verify_proposed_api, ApiError,
};

#[test]
fn pa01_signed_zero_and_large_integral_values_are_not_restricted_to_machine_indices() {
    for value in [
        0.0,
        -0.0,
        -1.0,
        1.0,
        -2147483649.0,
        2147483648.0,
        4294967296.0,
        9007199254740991.0,
        9007199254740992.0,
        f64::MAX,
    ] {
        assert_eq!(verify_integers(&[value]), Ok(()), "{value}");
    }
    assert_eq!(verify_integers(&[]), Ok(()));
}

#[test]
fn pa01_each_invalid_numeric_argument_rejects_with_source_message() {
    for value in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        0.5,
        -0.5,
        f64::MIN_POSITIVE,
    ] {
        for input in [vec![value], vec![1.0, value], vec![value, 1.0]] {
            let error = verify_integers(&input).unwrap_err();
            assert_eq!(error, ApiError::IntegerRequired);
            assert_eq!(error.to_string(), "This API only accepts integers");
        }
    }
}

#[test]
fn pa01_browser_positive_guard_retains_nan_and_zero_quirks() {
    for value in [f64::NAN, 0.0, -0.0, 1.0, 4294967296.0, f64::MAX] {
        assert_eq!(verify_positive_integers(&[value]), Ok(()));
    }
    for value in [-1.0, 0.5, -0.5, f64::INFINITY, f64::NEG_INFINITY] {
        let error = verify_positive_integers(&[value]).unwrap_err();
        assert_eq!(error, ApiError::PositiveIntegerRequired);
        assert_eq!(error.to_string(), "This API only accepts positive integers");
    }
}

#[test]
fn pa02_only_cols_and_rows_are_constructor_only_and_unicode_gate_is_dynamic() {
    for key in ["cols", "rows"] {
        let error = verify_mutable_option(key).unwrap_err();
        assert_eq!(
            error.to_string(),
            format!("Option \"{key}\" can only be set in the constructor")
        );
    }
    for key in [
        "",
        "Cols",
        "Rows",
        "allowProposedApi",
        "scrollback",
        "unknown",
    ] {
        assert_eq!(verify_mutable_option(key), Ok(()));
    }
    assert_eq!(verify_proposed_api(true), Ok(()));
    assert_eq!(
        verify_proposed_api(false).unwrap_err().to_string(),
        "You must set the allowProposedApi option to true to use proposed API"
    );
    assert_eq!(
        ApiError::NotOpened.to_string(),
        "Terminal must be opened first"
    );
}
