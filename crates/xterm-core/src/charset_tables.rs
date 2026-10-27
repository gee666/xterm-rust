// Copyright (c) 2016 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. See LICENSE.
// Maps transcribed from source_repo/src/common/data/Charsets.ts.

use super::Charset;
use std::collections::BTreeMap;
use std::sync::Arc;

// These are the xterm.js/vttest mappings, not alternative ISO 2022 tables.
const DEC_SPECIAL: &[(char, &str)] = &[
    ('`', "◆"),
    ('a', "▒"),
    ('b', "␉"),
    ('c', "␌"),
    ('d', "␍"),
    ('e', "␊"),
    ('f', "°"),
    ('g', "±"),
    ('h', "␤"),
    ('i', "␋"),
    ('j', "┘"),
    ('k', "┐"),
    ('l', "┌"),
    ('m', "└"),
    ('n', "┼"),
    ('o', "⎺"),
    ('p', "⎻"),
    ('q', "─"),
    ('r', "⎼"),
    ('s', "⎽"),
    ('t', "├"),
    ('u', "┤"),
    ('v', "┴"),
    ('w', "┬"),
    ('x', "│"),
    ('y', "≤"),
    ('z', "≥"),
    ('{', "π"),
    ('|', "≠"),
    ('}', "£"),
    ('~', "·"),
];
const BRITISH: &[(char, &str)] = &[('#', "£")];
const DUTCH: &[(char, &str)] = &[
    ('#', "£"),
    ('@', "¾"),
    ('[', "ij"),
    ('\\', "½"),
    (']', "|"),
    ('{', "¨"),
    ('|', "f"),
    ('}', "¼"),
    ('~', "´"),
];
const FINNISH: &[(char, &str)] = &[
    ('[', "Ä"),
    ('\\', "Ö"),
    (']', "Å"),
    ('^', "Ü"),
    ('`', "é"),
    ('{', "ä"),
    ('|', "ö"),
    ('}', "å"),
    ('~', "ü"),
];
const FRENCH: &[(char, &str)] = &[
    ('#', "£"),
    ('@', "à"),
    ('[', "°"),
    ('\\', "ç"),
    (']', "§"),
    ('{', "é"),
    ('|', "ù"),
    ('}', "è"),
    ('~', "¨"),
];
const FRENCH_CANADIAN: &[(char, &str)] = &[
    ('@', "à"),
    ('[', "â"),
    ('\\', "ç"),
    (']', "ê"),
    ('^', "î"),
    ('`', "ô"),
    ('{', "é"),
    ('|', "ù"),
    ('}', "è"),
    ('~', "û"),
];
const GERMAN: &[(char, &str)] = &[
    ('@', "§"),
    ('[', "Ä"),
    ('\\', "Ö"),
    (']', "Ü"),
    ('{', "ä"),
    ('|', "ö"),
    ('}', "ü"),
    ('~', "ß"),
];
const ITALIAN: &[(char, &str)] = &[
    ('#', "£"),
    ('@', "§"),
    ('[', "°"),
    ('\\', "ç"),
    (']', "é"),
    ('`', "ù"),
    ('{', "à"),
    ('|', "ò"),
    ('}', "è"),
    ('~', "ì"),
];
const NORWEGIAN_DANISH: &[(char, &str)] = &[
    ('@', "Ä"),
    ('[', "Æ"),
    ('\\', "Ø"),
    (']', "Å"),
    ('^', "Ü"),
    ('`', "ä"),
    ('{', "æ"),
    ('|', "ø"),
    ('}', "å"),
    ('~', "ü"),
];
const SPANISH: &[(char, &str)] = &[
    ('#', "£"),
    ('@', "§"),
    ('[', "¡"),
    ('\\', "Ñ"),
    (']', "¿"),
    ('{', "°"),
    ('|', "ñ"),
    ('}', "ç"),
];
const SWEDISH: &[(char, &str)] = &[
    ('@', "É"),
    ('[', "Ä"),
    ('\\', "Ö"),
    (']', "Å"),
    ('^', "Ü"),
    ('`', "é"),
    ('{', "ä"),
    ('|', "ö"),
    ('}', "å"),
    ('~', "ü"),
];
const SWISS: &[(char, &str)] = &[
    ('#', "ù"),
    ('@', "à"),
    ('[', "é"),
    ('\\', "ç"),
    (']', "ê"),
    ('^', "î"),
    ('_', "è"),
    ('`', "ô"),
    ('{', "ä"),
    ('|', "ö"),
    ('}', "ü"),
    ('~', "û"),
];

pub(super) fn build_charsets() -> BTreeMap<char, Charset> {
    let mut charsets = BTreeMap::new();
    for (aliases, entries) in [
        ("0", DEC_SPECIAL),
        ("A", BRITISH),
        ("4", DUTCH),
        ("C5", FINNISH),
        ("R", FRENCH),
        ("Q", FRENCH_CANADIAN),
        ("K", GERMAN),
        ("Y", ITALIAN),
        ("E6", NORWEGIAN_DANISH),
        ("Z", SPANISH),
        ("H7", SWEDISH),
        ("=", SWISS),
    ] {
        let map = Arc::new(
            entries
                .iter()
                .map(|&(key, value)| {
                    // All table keys are ASCII. UTF-16 conversion avoids a
                    // truncating cast if a future reference table changes.
                    let unit = key.encode_utf16(&mut [0; 2])[0];
                    (unit, value.encode_utf16().collect())
                })
                .collect(),
        );
        for alias in aliases.chars() {
            charsets.insert(alias, Arc::clone(&map));
        }
    }
    charsets
}
