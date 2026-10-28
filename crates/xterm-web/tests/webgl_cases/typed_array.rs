// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Faithful TypedArray.test.ts assertions and numeric edges.
use xterm_web::webgl::typed_array::{slice, slice_fallback};

macro_rules! original_kind {
    ($name:ident, $kind:ty, $large:expr) => {
        #[test]
        fn $name() {
            let data = [<$kind>::default(); 5];
            assert_eq!(
                slice_fallback(&data, Some(2.0), None).unwrap(),
                vec![<$kind>::default(); 3]
            );
            assert!(slice_fallback(&data, Some($large), None)
                .unwrap()
                .is_empty());
            assert_eq!(
                slice_fallback(&data, Some(-1.0), None).unwrap(),
                vec![<$kind>::default()]
            );
        }
    };
}
original_kind!(uint8_array, u8, 65535.0);
original_kind!(uint16_array, u16, 65535.0);
original_kind!(uint32_array, u32, 65537.0);
original_kind!(int8_array, i8, 65537.0);
original_kind!(int16_array, i16, 65535.0);
original_kind!(int32_array, i32, 65537.0);
original_kind!(float32_array, f32, 65537.0);
original_kind!(float64_array, f64, 65537.0);
// Clamped arrays already contain u8 values before slice; no input coercion is
// performed by either upstream slice implementation.
original_kind!(uint8_clamped_array, u8, 65537.0);

#[test]
fn original_start_cases() {
    let data = [1_u32, 2, 3, 4, 5];
    let expected: &[&[u32]] = &[
        &[5],
        &[1, 2, 3, 4, 5],
        &[2, 3, 4, 5],
        &[3, 4, 5],
        &[4, 5],
        &[5],
        &[],
    ];
    for (start, expected) in [-1.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0]
        .into_iter()
        .zip(expected)
    {
        assert_eq!(slice_fallback(&data, Some(start), None).unwrap(), *expected);
        assert_eq!(slice(&data, Some(start), None).unwrap(), *expected);
    }
}

#[test]
fn original_end_cases() {
    let data = [1_u32, 2, 3, 4, 5];
    let short: &[&[u32]] = &[&[], &[1, 2, 3], &[2, 3], &[3], &[], &[], &[]];
    let full: &[&[u32]] = &[
        &[5],
        &[1, 2, 3, 4, 5],
        &[2, 3, 4, 5],
        &[3, 4, 5],
        &[4, 5],
        &[5],
        &[],
    ];
    for end in [-2.0, 3.0, 8.0] {
        let expected = if end == 8.0 { full } else { short };
        for (start, expected) in [-1.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0]
            .into_iter()
            .zip(expected)
        {
            assert_eq!(
                slice_fallback(&data, Some(start), Some(end)).unwrap(),
                *expected
            );
            assert_eq!(slice(&data, Some(start), Some(end)).unwrap(), *expected);
        }
    }
}

#[test]
fn omitted_empty_and_independent_storage() {
    let mut data = [1_u32, 2, 3];
    let mut copied = slice(&data, None, None).unwrap();
    copied[0] = 99;
    assert_eq!(data, [1, 2, 3]);
    data[1] = 77;
    assert_eq!(copied, [99, 2, 3]);
    let mut fallback = slice_fallback(&data, None, None).unwrap();
    fallback[2] = 88;
    assert_eq!(data, [1, 77, 3]);
    assert!(slice::<u32>(&[], None, None).unwrap().is_empty());
    assert!(slice_fallback::<u32>(&[], Some(-1.0), Some(-2.0))
        .unwrap()
        .is_empty());
}

#[test]
fn fallback_preserves_legacy_negative_and_fractional_quirks() {
    let data = [1_u32, 2, 3, 4, 5];
    assert_eq!(
        slice_fallback(&data, Some(-7.0), None).unwrap(),
        [0, 0, 1, 2, 3, 4, 5]
    );
    assert_eq!(slice(&data, Some(-7.0), None).unwrap(), data);
    assert_eq!(
        slice_fallback(&data, Some(0.5), None).unwrap(),
        [0, 0, 0, 0]
    );
    assert_eq!(slice(&data, Some(0.5), None).unwrap(), data);
    assert_eq!(slice_fallback(&data, Some(1.5), Some(3.5)).unwrap(), [0, 0]);
    assert_eq!(slice(&data, Some(1.5), Some(3.5)).unwrap(), [2, 3]);
}

#[test]
fn nonfinite_indexes_follow_the_selected_source_operation() {
    let data = [1_u32, 2, 3];
    assert_eq!(slice(&data, Some(f64::NAN), None).unwrap(), data);
    assert!(slice_fallback(&data, Some(f64::NAN), None)
        .unwrap()
        .is_empty());
    assert_eq!(slice(&data, Some(f64::NEG_INFINITY), None).unwrap(), data);
    assert!(slice_fallback(&data, Some(f64::NEG_INFINITY), None)
        .unwrap()
        .is_empty());
    assert!(slice(&data, Some(f64::INFINITY), None).unwrap().is_empty());
    assert!(slice_fallback(&data, Some(f64::INFINITY), None)
        .unwrap()
        .is_empty());
    assert!(slice(&data, None, Some(f64::NAN)).unwrap().is_empty());
    assert!(slice_fallback(&data, None, Some(f64::NAN))
        .unwrap()
        .is_empty());
}

#[test]
fn floating_values_signed_zero_and_undefined_assignment() {
    let data = [f32::NEG_INFINITY, -0.0, 0.25, f32::INFINITY, f32::NAN];
    let copied = slice(&data, Some(1.0), Some(4.0)).unwrap();
    assert_eq!(
        copied
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        [
            (-0.0_f32).to_bits(),
            0.25_f32.to_bits(),
            f32::INFINITY.to_bits()
        ]
    );
    let fallback = slice_fallback(&data, Some(0.5), None).unwrap();
    assert_eq!(fallback.len(), 4);
    assert!(fallback.iter().all(|value| value.is_nan()));
    let data = [0.1_f64, -0.0, f64::NAN];
    let copied = slice(&data, None, None).unwrap();
    assert_eq!(copied[0].to_bits(), data[0].to_bits());
    assert_eq!(copied[1].to_bits(), data[1].to_bits());
    assert!(copied[2].is_nan());
}
