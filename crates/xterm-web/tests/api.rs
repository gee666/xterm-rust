use xterm_web::api::{
    verify_integers, verify_mutable_option, verify_positive_integers, verify_proposed_api, ApiError,
};

#[test]
fn integer_boundary_matches_number_semantics() {
    for value in [0.0, -0.0, 1.0, -1.0, 9_007_199_254_740_992.0, f64::MAX] {
        assert_eq!(verify_integers(&[value]), Ok(()), "{value}");
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.5, -0.5] {
        assert_eq!(verify_integers(&[value]), Err(ApiError::IntegerRequired));
    }
    assert_eq!(verify_integers(&[]), Ok(()));
    assert_eq!(
        verify_integers(&[1.0, 2.5, 3.0]),
        Err(ApiError::IntegerRequired)
    );
}

#[test]
fn positive_validator_preserves_upstream_truthiness_guard() {
    for value in [f64::NAN, 0.0, -0.0, 1.0, f64::MAX] {
        assert_eq!(verify_positive_integers(&[value]), Ok(()), "{value}");
    }
    for value in [f64::INFINITY, f64::NEG_INFINITY, -1.0, 0.5, -0.5] {
        assert_eq!(
            verify_positive_integers(&[value]),
            Err(ApiError::PositiveIntegerRequired)
        );
    }
}

#[test]
fn constructor_only_and_proposed_errors_match_upstream() {
    for key in ["rows", "cols"] {
        assert_eq!(
            verify_mutable_option(key).unwrap_err().to_string(),
            format!("Option \"{key}\" can only be set in the constructor")
        );
    }
    for key in ["fontSize", "Rows", "allowProposedApi", "unknown"] {
        assert_eq!(verify_mutable_option(key), Ok(()));
    }
    assert_eq!(verify_proposed_api(true), Ok(()));
    assert_eq!(
        verify_proposed_api(false).unwrap_err().to_string(),
        "You must set the allowProposedApi option to true to use proposed API"
    );
    assert_eq!(
        ApiError::IntegerRequired.to_string(),
        "This API only accepts integers"
    );
    assert_eq!(
        ApiError::PositiveIntegerRequired.to_string(),
        "This API only accepts positive integers"
    );
}
