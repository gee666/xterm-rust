// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of addon-program-status/ProgramStatus.ts.

use super::{StatusKind, StatusRecord, StatusReport, StatusState};

const PAYLOAD_LIMIT: usize = 4087;
const KEY_LIMIT: usize = 16;
const MESSAGE_ENCODED_LIMIT: usize = 2732;
const MESSAGE_DECODED_LIMIT: usize = 2048;
const TITLE_ENCODED_LIMIT: usize = 256;
const TITLE_DECODED_LIMIT: usize = 192;
const SEGMENT_LIMIT: usize = 32;
const ID_LIMIT: usize = 128;
const ID_LEVEL_LIMIT: usize = 8;

/// Lone surrogates cannot occur in returned fields. Lossy decoding preserves
/// their upstream UTF-8 byte count and their rejection by ASCII field grammars.
pub fn parse_report_utf16(data: &[u16]) -> Option<StatusReport> {
    if data.len() > PAYLOAD_LIMIT {
        return None;
    }
    parse_report(&String::from_utf16_lossy(data))
}

/// `None` means the complete report must be discarded, not that fields were absent.
pub fn parse_report(data: &str) -> Option<StatusReport> {
    if data.len() > PAYLOAD_LIMIT {
        return None;
    }
    let mut id = "";
    let mut state = "";
    let mut kind = "";
    let mut progress = "";
    let mut app = None;
    let mut title = None;
    let mut message = None;
    for pair in data.split(':') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        let key = trim(key);
        let value = trim(value);
        if key.len() > KEY_LIMIT {
            return None;
        }
        if key.is_empty() || !key.bytes().all(|b| b.is_ascii_lowercase()) {
            continue;
        }
        let limit = match key {
            "msg" => Some(MESSAGE_ENCODED_LIMIT),
            "title" => Some(TITLE_ENCODED_LIMIT),
            "app" => Some(SEGMENT_LIMIT),
            "id" => Some(ID_LIMIT),
            _ => None,
        };
        if limit.is_some_and(|limit| value.len() > limit) || (key == "id" && !is_id(value)) {
            return None;
        }
        if !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.,+/=-".contains(&b))
        {
            continue;
        }
        match key {
            "id" => id = value,
            "state" => state = value,
            "kind" => kind = value,
            "progress" => progress = value,
            "app" => app = Some(value),
            "title" => title = Some(decode_text(value, TITLE_DECODED_LIMIT)?),
            "msg" => message = Some(decode_text(value, MESSAGE_DECODED_LIMIT)?),
            _ => {}
        }
    }
    if state == "clear" {
        return Some(StatusReport::Clear(id.to_owned()));
    }
    let state = match state {
        "idle" => StatusState::Idle,
        "working" => StatusState::Working,
        "done" => StatusState::Done,
        "blocked" => StatusState::Blocked,
        "error" => StatusState::Error,
        _ => return None,
    };
    let mut record = StatusRecord::new(id, state);
    if state == StatusState::Blocked {
        record.kind = match kind {
            "permission" => Some(StatusKind::Permission),
            "question" => Some(StatusKind::Question),
            "auth" => Some(StatusKind::Auth),
            _ => None,
        };
    }
    if matches!(state, StatusState::Working | StatusState::Blocked) {
        record.progress = parse_progress(progress);
    }
    record.app = app.filter(|app| is_segment(app)).map(str::to_owned);
    record.title = title;
    record.message = message;
    Some(StatusReport::Record(record))
}

fn trim(s: &str) -> &str {
    s.trim_matches(|c| matches!(c, ' ' | '\t'..='\r'))
}

fn is_segment(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= SEGMENT_LIMIT
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.+-".contains(&b))
}

fn is_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= ID_LIMIT
        && s.split('/').count() <= ID_LEVEL_LIMIT
        && s.split('/').all(is_segment)
}

fn parse_progress(s: &str) -> Option<u8> {
    if s.is_empty() {
        return None;
    }
    let mut n = 0_u8;
    for b in s.bytes() {
        if !b.is_ascii_digit() {
            return None;
        }
        n = n.checked_mul(10)?.checked_add(b - b'0')?;
        if n > 100 {
            return None;
        }
    }
    Some(n)
}

// Upstream rejects noncanonical trailing bits and accepts optional padding.
// This small decoder avoids a dependency just for this fixed wire grammar.
fn decode_text(value: &str, limit: usize) -> Option<String> {
    let content_len = value.find('=').unwrap_or(value.len());
    let padding = value.len() - content_len;
    let remainder = content_len % 4;
    if remainder == 1
        || padding > 2
        || !value[content_len..].bytes().all(|b| b == b'=')
        || (padding != 0 && (value.len() % 4 != 0 || padding != 4 - remainder))
    {
        return None;
    }
    let mut bytes = Vec::with_capacity((content_len * 3 / 4).min(limit));
    let mut bits = 0_u32;
    let mut count = 0;
    for b in value[..content_len].bytes() {
        let sextet = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        };
        bits = (bits << 6) | u32::from(sextet);
        count += 6;
        if count >= 8 {
            count -= 8;
            if bytes.len() == limit {
                return None;
            }
            bytes.push(((bits >> count) & 255) as u8);
            bits &= (1 << count) - 1;
        }
    }
    if bits != 0 {
        return None;
    }
    let text = String::from_utf8(bytes).ok()?;
    if text
        .chars()
        .any(|c| c <= '\u{1f}' || ('\u{7f}'..='\u{9f}').contains(&c))
    {
        return None;
    }
    Some(text)
}
