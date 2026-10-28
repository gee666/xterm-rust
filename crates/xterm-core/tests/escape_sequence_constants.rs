// Copyright (c) 2017 The xterm.js authors. All rights reserved. Licensed under MIT.
use xterm_core::data::escape_sequences::{c0, c1, c1_escaped};

#[test]
fn every_named_c0_string_has_its_original_single_code_unit() {
    let values = [
        c0::NUL,
        c0::SOH,
        c0::STX,
        c0::ETX,
        c0::EOT,
        c0::ENQ,
        c0::ACK,
        c0::BEL,
        c0::BS,
        c0::HT,
        c0::LF,
        c0::VT,
        c0::FF,
        c0::CR,
        c0::SO,
        c0::SI,
        c0::DLE,
        c0::DC1,
        c0::DC2,
        c0::DC3,
        c0::DC4,
        c0::NAK,
        c0::SYN,
        c0::ETB,
        c0::CAN,
        c0::EM,
        c0::SUB,
        c0::ESC,
        c0::FS,
        c0::GS,
        c0::RS,
        c0::US,
        c0::SP,
        c0::DEL,
    ];
    assert_eq!(values.len(), 34);
    for (index, value) in values.into_iter().enumerate() {
        let expected = if index == 33 { 0x7f } else { index as u16 };
        assert_eq!(
            value.encode_utf16().collect::<Vec<_>>(),
            [expected],
            "C0 index{index}"
        );
        assert_eq!(value.as_bytes(), [expected as u8]);
    }
    assert_eq!(c0::SP, " ");
    assert_eq!(c0::CR.as_bytes(), [13]);
    assert_eq!(c0::LF.as_bytes(), [10]);
}

#[test]
fn every_named_c1_string_is_a_single_utf16_unit_not_a_raw_utf8_byte() {
    let values = [
        c1::PAD,
        c1::HOP,
        c1::BPH,
        c1::NBH,
        c1::IND,
        c1::NEL,
        c1::SSA,
        c1::ESA,
        c1::HTS,
        c1::HTJ,
        c1::VTS,
        c1::PLD,
        c1::PLU,
        c1::RI,
        c1::SS2,
        c1::SS3,
        c1::DCS,
        c1::PU1,
        c1::PU2,
        c1::STS,
        c1::CCH,
        c1::MW,
        c1::SPA,
        c1::EPA,
        c1::SOS,
        c1::SGCI,
        c1::SCI,
        c1::CSI,
        c1::ST,
        c1::OSC,
        c1::PM,
        c1::APC,
    ];
    assert_eq!(values.len(), 32);
    for (index, value) in values.into_iter().enumerate() {
        let expected = 0x80 + index as u16;
        assert_eq!(
            value.encode_utf16().collect::<Vec<_>>(),
            [expected],
            "C1 index{index}"
        );
        assert_eq!(value.as_bytes(), [0xc2, expected as u8]);
    }
}

#[test]
fn escaped_st_retains_two_units_and_remains_distinct_from_c1_st() {
    assert_eq!(
        c1_escaped::ST.encode_utf16().collect::<Vec<_>>(),
        [0x1b, 0x5c]
    );
    assert_eq!(c1_escaped::ST.as_bytes(), [0x1b, 0x5c]);
    assert_eq!(c1::ST.encode_utf16().collect::<Vec<_>>(), [0x9c]);
    assert_ne!(c1_escaped::ST, c1::ST);
}
