// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{SearchError, SearchOptions};
use std::borrow::Cow;

const NON_WORD_CHARACTERS: &str = " ~!@#$%^&*()+`-=[]{}|\\;:\"',./<>?";

pub(super) fn find(
    term: &[u16],
    line: &[u16],
    offset: usize,
    options: &SearchOptions,
    reverse: bool,
) -> Result<Option<(usize, Vec<u16>)>, SearchError> {
    let insensitive = options.case_sensitive != Some(true);
    let regex = options.regex == Some(true);
    let mut matched_term = None;
    let search_line = if insensitive && !regex {
        Cow::Owned(lowercase(line))
    } else {
        Cow::Borrowed(line)
    };
    let index = if regex {
        let flags = regress::Flags {
            icase: insensitive,
            ..regress::Flags::default()
        };
        // The source never passes the Unicode flag. Parse and match individual
        // UTF-16 code units so a dot may match one half of a surrogate pair.
        let re = regress::Regex::from_unicode(term.iter().copied().map(u32::from), flags)
            .map_err(|e| SearchError::InvalidRegex(e.to_string()))?;
        let slice = if reverse {
            &line[..offset.min(line.len())]
        } else {
            &line[offset.min(line.len())..]
        };
        let mut found = None;
        let mut start = 0;
        while start <= slice.len() {
            let Some(m) = re.find_from_ucs2(slice, start).next() else {
                break;
            };
            if !reverse && m.start() == m.end() {
                break;
            }
            matched_term = Some(&slice[m.range()]);
            found = Some(m.start() + if reverse { 0 } else { offset });
            if !reverse {
                break;
            }
            // JS rewinds lastIndex to match start + 1, allowing overlapping
            // matches and making progress even for a zero-length match.
            start = m.start() + 1;
        }
        found
    } else {
        let search_term = if insensitive {
            Cow::Owned(lowercase(term))
        } else {
            Cow::Borrowed(term)
        };
        if reverse {
            if offset < search_term.len() {
                None
            } else {
                search_line
                    .windows(search_term.len())
                    .enumerate()
                    .take(offset - search_term.len() + 1)
                    .filter(|(_, value)| *value == search_term.as_ref())
                    .map(|(i, _)| i)
                    .next_back()
            }
        } else {
            search_line
                .get(offset..)
                .and_then(|s| {
                    s.windows(search_term.len())
                        .position(|w| w == search_term.as_ref())
                })
                .map(|index| offset + index)
        }
    };
    Ok(index.and_then(|i| {
        let matched = matched_term.unwrap_or(term);
        if options.whole_word == Some(true) && !is_whole_word(i, &search_line, matched) {
            None
        } else {
            Some((i, matched.to_vec()))
        }
    }))
}

fn is_whole_word(index: usize, line: &[u16], term: &[u16]) -> bool {
    let delimiter = |unit: Option<&u16>| {
        unit.is_some_and(|&u| u < 128 && NON_WORD_CHARACTERS.as_bytes().contains(&(u as u8)))
    };
    (index == 0 || delimiter(line.get(index - 1)))
        && (index + term.len() == line.len() || delimiter(line.get(index + term.len())))
}

fn lowercase(value: &[u16]) -> Vec<u16> {
    let mut output = Vec::new();
    let mut valid = String::new();
    for decoded in char::decode_utf16(value.iter().copied()) {
        match decoded {
            Ok(c) => valid.push(c),
            Err(e) => {
                output.extend(valid.to_lowercase().encode_utf16());
                valid.clear();
                output.push(e.unpaired_surrogate());
            }
        }
    }
    output.extend(valid.to_lowercase().encode_utf16());
    output
}
