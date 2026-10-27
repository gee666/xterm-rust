// Copyright (c) 2021 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Assertions ported from XParseColor.test.ts.

use xterm_core::input::xparse_color::{parse_color, to_rgb_string};

#[test]
fn rgb_scheme_in_4_8_12_16_bit() {
    for width in 1..=4 {
        for (digits, expected) in [('0', [0; 3]), ('f', [255; 3])] {
            let channel = digits.to_string().repeat(width);
            assert_eq!(
                parse_color(&format!("rgb:{channel}/{channel}/{channel}")),
                Some(expected)
            );
        }
        let channels: Vec<_> = ['1', '2', '3'].map(|c| c.to_string().repeat(width)).into();
        assert_eq!(
            parse_color(&format!("rgb:{}", channels.join("/"))),
            Some([17, 34, 51])
        );
    }
}

#[test]
fn hash_scheme_in_4_8_12_16_bit() {
    for width in 1..=4 {
        assert_eq!(
            parse_color(&format!("#{}", "0".repeat(3 * width))),
            Some([0; 3])
        );
        assert_eq!(
            parse_color(&format!("#{}", "f".repeat(3 * width))),
            Some([if width == 1 { 240 } else { 255 }; 3])
        );
        let channels: String = ['1', '2', '3']
            .map(|c| c.to_string().repeat(width))
            .concat();
        assert_eq!(
            parse_color(&format!("#{channels}")),
            Some(if width == 1 {
                [16, 32, 48]
            } else {
                [17, 34, 51]
            })
        );
    }
}

#[test]
fn supports_upper_case() {
    assert_eq!(parse_color("RGB:0/A/F"), Some([0, 170, 255]));
    assert_eq!(parse_color("#FFF"), Some([240; 3]));
}

#[test]
fn does_not_parse_illegal_combinations() {
    for spec in [
        "rgb:0/11/222",
        "rgbi:00/11/22",
        "#aabbbcc",
        "#aabbgg",
        "rgb:aa/bb/gg",
    ] {
        assert_eq!(parse_color(spec), None, "{spec}");
    }
}

#[test]
fn format_rgb_scheme_in_4_8_12_16_bit() {
    for width in 1..=4 {
        for channel in ['0', 'f'] {
            let channel = channel.to_string().repeat(width);
            let spec = format!("rgb:{channel}/{channel}/{channel}");
            assert_eq!(
                to_rgb_string(parse_color(&spec).unwrap(), Some(4 * width as u32)),
                spec
            );
        }
        let channels: Vec<_> = ['1', '2', '3'].map(|c| c.to_string().repeat(width)).into();
        let spec = format!("rgb:{}", channels.join("/"));
        assert_eq!(
            to_rgb_string(parse_color(&spec).unwrap(), Some(4 * width as u32)),
            spec
        );
    }
}

#[test]
fn defaults_to_16_bit_output() {
    for spec in ["rgb:1/2/3", "rgb:11/22/33", "rgb:111/222/333"] {
        assert_eq!(
            to_rgb_string(parse_color(spec).unwrap(), None),
            "rgb:1111/2222/3333"
        );
    }
    assert_eq!(
        to_rgb_string(parse_color("rgb:123/123/123").unwrap(), None),
        "rgb:1212/1212/1212"
    );
}

#[test]
fn reduces_colors_to_8_bit_resolution() {
    assert_eq!(
        to_rgb_string(parse_color("rgb:123/123/123").unwrap(), Some(12)),
        "rgb:121/121/121"
    );
    assert_eq!(
        to_rgb_string(parse_color("rgb:1234/1234/1234").unwrap(), Some(16)),
        "rgb:1212/1212/1212"
    );
}

#[test]
fn rejects_non_ascii_and_whitespace() {
    for spec in [
        "",
        "red",
        "#",
        "#１２３",
        "#12😀",
        " rgb:1/2/3",
        "rgb:1/2/3\n",
        "rgb:/1/2",
        "rgb:1/2/3/4",
        "#123\n",
        "rgb:α/β/γ",
        "rgb:00000/00000/00000",
    ] {
        assert_eq!(parse_color(spec), None, "{spec:?}");
    }
}

#[test]
fn format_low_channels_and_unrecognized_depths_match_upstream() {
    assert_eq!(to_rgb_string([0, 1, 15], Some(4)), "rgb:0/1/f");
    assert_eq!(to_rgb_string([0, 1, 15], Some(8)), "rgb:00/01/0f");
    assert_eq!(to_rgb_string([0, 1, 15], Some(12)), "rgb:000/010/0f0");
    for depth in [0, 1, 15, 17, u32::MAX] {
        assert_eq!(to_rgb_string([0, 1, 15], Some(depth)), "rgb:0000/0101/0f0f");
    }
}
