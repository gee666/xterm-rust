use super::{prefix, range};

#[test]
fn source_output_windows_preserve_backreferences_and_declared_output_limits() {
    for input in [
        include_bytes!("../../tests/fixtures/cff-window-stored.deflate").as_slice(),
        include_bytes!("../../tests/fixtures/cff-window-fixed.deflate").as_slice(),
        include_bytes!("../../tests/fixtures/cff-window-dynamic.deflate").as_slice(),
    ] {
        for limit in [0, 1, 32767, 32768, 32769, 70000, 80000] {
            let mut full = vec![0; limit];
            prefix(input, &mut full).unwrap();
            for (i, &actual) in full.iter().enumerate() {
                assert_eq!(actual, if i < 70000 { (i % 256) as u8 } else { 0 });
            }
            for (start, count) in [(0, 16), (32750, 64), (65520, 32), (69990, 32), (79980, 32)] {
                let mut actual = vec![0; count];
                range(input, limit, start, &mut actual).unwrap();
                for (i, &actual) in actual.iter().enumerate() {
                    let position = start + i;
                    assert_eq!(
                        actual,
                        if position < limit && position < 70000 {
                            (position % 256) as u8
                        } else {
                            0
                        },
                        "limit{limit}/start{start}/index{i}"
                    );
                }
            }
        }
    }
}
