// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Generated from original literal example assertions.
use super::*;

// Original example: CSI with print and execute
#[test]
fn example_00() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("\u{1b}[<31;5mHello World! öäü€\u{a}abc");
    assert_eq!(
        fixture.events(),
        vec![
            Event::Csi("<m".into(), vec![V::Param(31), V::Param(5)]),
            Event::Print("Hello World! öäü€".into()),
            Event::Execute(10),
            Event::Print("abc".into())
        ]
    );
}

// Original example: OSC
#[test]
fn example_01() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("\u{1b}]0;abc123€öäü\u{7}");
    assert_eq!(
        fixture.events(),
        vec![Event::Osc("0;abc123€öäü, success: true".into())]
    );
}

// Original example: single DCS
#[test]
fn example_02() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("\u{1b}P1;2;3+$aäbc;däe");
    assert_eq!(
        fixture.events(),
        vec![
            Event::DcsHook(vec![V::Param(1), V::Param(2), V::Param(3)]),
            Event::DcsPut("äbc;däe".into()),
            Event::DcsEnd(true)
        ]
    );
}

// Original example: multi DCS
#[test]
fn example_03() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("\u{1b}P1;2;3+$abc;de");
    assert_eq!(
        fixture.events(),
        vec![
            Event::DcsHook(vec![V::Param(1), V::Param(2), V::Param(3)]),
            Event::DcsPut("bc;de".into())
        ]
    );
    fixture.clear();
    fixture.parse("abc");
    assert_eq!(
        fixture.events(),
        vec![Event::DcsPut("abc".into()), Event::DcsEnd(true)]
    );
}

// Original example: print + DCS(C1)
#[test]
fn example_04() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("abc1;2;3+$abc;de");
    assert_eq!(
        fixture.events(),
        vec![
            Event::Print("abc".into()),
            Event::DcsHook(vec![V::Param(1), V::Param(2), V::Param(3)]),
            Event::DcsPut("bc;de".into()),
            Event::DcsEnd(true)
        ]
    );
}

// Original example: print + PM(C1) + print
#[test]
fn example_05() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("abc123tzfdefg");
    assert_eq!(
        fixture.events(),
        vec![Event::Print("abc".into()), Event::Print("defg".into())]
    );
}

// Original example: print + OSC(C1) + print
#[test]
fn example_06() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("abc123;tzfdefg");
    assert_eq!(
        fixture.events(),
        vec![
            Event::Print("abc".into()),
            Event::Osc("123;tzf, success: true".into()),
            Event::Print("defg".into())
        ]
    );
}

// Original example: single APC
#[test]
fn example_07() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("\u{1b}_X3+$aäbc;däe");
    assert_eq!(
        fixture.events(),
        vec![
            Event::ApcStart,
            Event::ApcPut("3+$aäbc;däe".into()),
            Event::ApcEnd(true)
        ]
    );
}

// Original example: multi APC
#[test]
fn example_08() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("\u{1b}_Xabc;de");
    assert_eq!(
        fixture.events(),
        vec![Event::ApcStart, Event::ApcPut("abc;de".into())]
    );
    fixture.clear();
    fixture.parse("abc");
    assert_eq!(
        fixture.events(),
        vec![Event::ApcPut("abc".into()), Event::ApcEnd(true)]
    );
}

// Original example: print + DCS(C1) + print
#[test]
fn example_09() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("abcAbc;dexyz");
    assert_eq!(
        fixture.events(),
        vec![
            Event::Print("abc".into()),
            Event::ApcStart,
            Event::ApcPut("bc;de".into()),
            Event::ApcEnd(true),
            Event::Print("xyz".into())
        ]
    );
}

// Original example: print + DCS(C0) + print
#[test]
fn example_10() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("abc\u{1b}_Abc;de\u{1b}\\xyz");
    assert_eq!(
        fixture.events(),
        vec![
            Event::Print("abc".into()),
            Event::ApcStart,
            Event::ApcPut("bc;de".into()),
            Event::ApcEnd(true),
            Event::Print("xyz".into())
        ]
    );
}

// Original example: error recovery
#[test]
fn example_11() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("\u{1b}[1€abcdefg<;c");
    assert_eq!(
        fixture.events(),
        vec![
            Event::Print("abcdefg".into()),
            Event::Csi("<c".into(), vec![V::Param(0), V::Param(0)])
        ]
    );
}

// Original example: 7bit ST should be swallowed
#[test]
fn example_12() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("abc123;tzf\u{1b}\\defg");
    assert_eq!(
        fixture.events(),
        vec![
            Event::Print("abc".into()),
            Event::Osc("123;tzf, success: true".into()),
            Event::Print("defg".into())
        ]
    );
}

// Original example: colon notation in CSI params
#[test]
fn example_13() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("\u{1b}[<31;5::123:;8mHello World! öäü€\u{a}abc");
    assert_eq!(
        fixture.events(),
        vec![
            Event::Csi(
                "<m".into(),
                vec![
                    V::Param(31),
                    V::Param(5),
                    V::SubParams(vec![-1, 123, -1]),
                    V::Param(8)
                ]
            ),
            Event::Print("Hello World! öäü€".into()),
            Event::Execute(10),
            Event::Print("abc".into())
        ]
    );
}

// Original example: colon notation in DCS params
#[test]
fn example_14() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("abc1;2::55;3+$abc;de");
    assert_eq!(
        fixture.events(),
        vec![
            Event::Print("abc".into()),
            Event::DcsHook(vec![
                V::Param(1),
                V::Param(2),
                V::SubParams(vec![-1, 55]),
                V::Param(3)
            ]),
            Event::DcsPut("bc;de".into()),
            Event::DcsEnd(true)
        ]
    );
}

// Original example: CAN should abort DCS
#[test]
fn example_15() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("abc1;2::55;3+$abc;de\u{18}");
    assert_eq!(
        fixture.events(),
        vec![
            Event::Print("abc".into()),
            Event::DcsHook(vec![
                V::Param(1),
                V::Param(2),
                V::SubParams(vec![-1, 55]),
                V::Param(3)
            ]),
            Event::DcsPut("bc;de".into()),
            Event::DcsEnd(false)
        ]
    );
}

// Original example: SUB should abort DCS
#[test]
fn example_16() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("abc1;2::55;3+$abc;de\u{1a}");
    assert_eq!(
        fixture.events(),
        vec![
            Event::Print("abc".into()),
            Event::DcsHook(vec![
                V::Param(1),
                V::Param(2),
                V::SubParams(vec![-1, 55]),
                V::Param(3)
            ]),
            Event::DcsPut("bc;de".into()),
            Event::DcsEnd(false)
        ]
    );
}

// Original example: CAN should abort APC
#[test]
fn example_17() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("abcXbc;de\u{18}");
    assert_eq!(
        fixture.events(),
        vec![
            Event::Print("abc".into()),
            Event::ApcStart,
            Event::ApcPut("bc;de".into()),
            Event::ApcEnd(false)
        ]
    );
}

// Original example: SUB should abort APC
#[test]
fn example_18() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("abcXbc;de\u{1a}");
    assert_eq!(
        fixture.events(),
        vec![
            Event::Print("abc".into()),
            Event::ApcStart,
            Event::ApcPut("bc;de".into()),
            Event::ApcEnd(false)
        ]
    );
}

// Original example: CAN should abort OSC
#[test]
fn example_19() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("\u{1b}]0;abc123€öäü\u{18}");
    assert_eq!(
        fixture.events(),
        vec![Event::Osc("0;abc123€öäü, success: false".into())]
    );
}

// Original example: SUB should abort OSC
#[test]
fn example_20() {
    let mut fixture = Fixture::new();
    fixture.parser.reset().unwrap();
    fixture.clear();
    fixture.parse("\u{1b}]0;abc123€öäü\u{1a}");
    assert_eq!(
        fixture.events(),
        vec![Event::Osc("0;abc123€öäü, success: false".into())]
    );
}
