// Copyright (c) 2023 The xterm.js authors. MIT license.
//! Streaming IIP header parser. Source separator and marker quirks are intentional.
use super::header_codec::{decode, NameDecodePolicy};
use std::fmt;

pub const MAX_FIELD_CHARS: usize = 1024;
const FILE: &[u32] = &[70, 105, 108, 101];
const FILE_PART: &[u32] = &[70, 105, 108, 101, 80, 97, 114, 116];
const MULTIPART: &[u32] = &[77, 117, 108, 116, 105, 112, 97, 114, 116, 70, 105, 108, 101];
const FILE_END: &[u32] = &[70, 105, 108, 101, 69, 110, 100];
const REPORT_CELL_SIZE: &[u32] = &[
    82, 101, 112, 111, 114, 116, 67, 101, 108, 108, 83, 105, 122, 101,
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum HeaderState {
    #[default]
    Start = 0,
    Abort = 1,
    Key = 2,
    Value = 3,
    End = 4,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SequenceType {
    Invalid = 0,
    File = 1,
    MultipartFile = 2,
    FilePart = 3,
    FileEnd = 4,
    ReportCellSize = 5,
}
#[derive(Clone, Debug, PartialEq)]
pub enum HeaderValue {
    Number(f64),
    Text(Vec<u16>),
    Codepoints(Vec<u32>),
    Null,
    Undefined,
    Boolean(bool),
    DecoderObject,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HeaderFields {
    entries: Vec<(Vec<u16>, HeaderValue)>,
    null_prototype: bool,
}
impl HeaderFields {
    pub fn get(&self, key: &[u16]) -> Option<&HeaderValue> {
        self.entries
            .iter()
            .find(|entry| entry.0 == key)
            .map(|entry| &entry.1)
    }
    pub fn get_ascii(&self, key: &str) -> Option<&HeaderValue> {
        self.get(&key.encode_utf16().collect::<Vec<_>>())
    }
    pub fn set(&mut self, key: Vec<u16>, value: HeaderValue) {
        if key == [95, 95, 112, 114, 111, 116, 111, 95, 95] && !self.null_prototype {
            if value == HeaderValue::Null {
                self.null_prototype = true;
            }
            return;
        }
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.0 == key) {
            entry.1 = value;
        } else {
            self.entries.push((key, value));
        }
    }
    /// Own keys follow ECMAScript integer-index ordering before other insertions.
    pub fn entries(&self) -> Vec<(&[u16], &HeaderValue)> {
        let mut entries: Vec<_> = self
            .entries
            .iter()
            .map(|(key, value)| (key.as_slice(), value))
            .collect();
        entries.sort_by_key(|(key, _)| array_index(key).map_or((1, 0), |index| (0, index)));
        entries
    }
    pub fn has_null_prototype(&self) -> bool {
        self.null_prototype
    }
    fn set_type(&mut self, kind: SequenceType) {
        self.set(
            "type".encode_utf16().collect(),
            HeaderValue::Number(f64::from(kind as u8)),
        );
    }
}
fn array_index(key: &[u16]) -> Option<u32> {
    if key.is_empty() || (key.len() > 1 && key[0] == 48) {
        return None;
    }
    let mut value = 0_u32;
    for &unit in key {
        if !(48..=57).contains(&unit) {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u32::from(unit - 48))?;
    }
    (value != u32::MAX).then_some(value)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseOutcome {
    NeedMore,
    Rejected,
    Payload(usize),
}
impl ParseOutcome {
    pub fn source_code(self) -> Result<i64, SourceCodeError> {
        match self {
            Self::NeedMore => Ok(-2),
            Self::Rejected => Ok(-1),
            Self::Payload(offset) => i64::try_from(offset).map_err(|_| SourceCodeError),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceCodeError;
impl fmt::Display for SourceCodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("IIP payload offset exceeds the source-code integer range")
    }
}
impl std::error::Error for SourceCodeError {}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeaderRangeError;
impl fmt::Display for HeaderRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid IIP input slice range")
    }
}
impl std::error::Error for HeaderRangeError {}

pub struct HeaderParser {
    pub state: HeaderState,
    pub fields: HeaderFields,
    buffer: [u32; MAX_FIELD_CHARS],
    position: usize,
    key: Vec<u16>,
    policy: NameDecodePolicy,
}
impl Default for HeaderParser {
    fn default() -> Self {
        Self::new(NameDecodePolicy::NodeBuffer)
    }
}
impl HeaderParser {
    pub fn new(policy: NameDecodePolicy) -> Self {
        Self {
            state: HeaderState::Start,
            fields: HeaderFields::default(),
            buffer: [0; MAX_FIELD_CHARS],
            position: 0,
            key: Vec::new(),
            policy,
        }
    }
    pub fn reset(&mut self) {
        self.buffer.fill(0);
        self.state = HeaderState::Start;
        self.position = 0;
        self.fields = HeaderFields::default();
        self.key.clear();
    }
    pub fn position(&self) -> usize {
        self.position
    }
    pub fn current_key(&self) -> &[u16] {
        &self.key
    }
    pub fn buffer(&self) -> &[u32; MAX_FIELD_CHARS] {
        &self.buffer
    }
    pub fn end(&mut self) -> ParseOutcome {
        if self.state == HeaderState::Start {
            let kind =
                if self.position == FILE_END.len() && self.buffer[..FILE_END.len()] == *FILE_END {
                    Some(SequenceType::FileEnd)
                } else if self.position == REPORT_CELL_SIZE.len()
                    && self.buffer[..REPORT_CELL_SIZE.len()] == *REPORT_CELL_SIZE
                {
                    Some(SequenceType::ReportCellSize)
                } else {
                    None
                };
            if let Some(kind) = kind {
                self.fields.set_type(kind);
                self.state = HeaderState::End;
                return ParseOutcome::Payload(0);
            }
            return self.abort();
        }
        if self.state == HeaderState::End {
            return ParseOutcome::Payload(0);
        }
        if self.state == HeaderState::Value
            && self.fields.get_ascii("type") == Some(&HeaderValue::Number(2.0))
        {
            if !self.store_value(self.position) {
                return self.abort();
            }
            self.state = HeaderState::End;
            return ParseOutcome::Payload(0);
        }
        self.abort()
    }
    /// Bounds are validated at the Rust API boundary; parser-provided offsets are absolute.
    pub fn parse(
        &mut self,
        data: &[u32],
        start: usize,
        end: usize,
    ) -> Result<ParseOutcome, HeaderRangeError> {
        if start > end || end > data.len() {
            return Err(HeaderRangeError);
        }
        let mut state = self.state;
        let mut position = self.position;
        if matches!(state, HeaderState::Abort | HeaderState::End)
            || (state == HeaderState::Start && position > 14)
        {
            return Ok(ParseOutcome::Rejected);
        }
        for (index, &code) in data.iter().enumerate().take(end).skip(start) {
            match code {
                59 => {
                    if !self.store_value(position) {
                        return Ok(self.abort());
                    }
                    state = HeaderState::Key;
                    position = 0;
                }
                61 => match state {
                    HeaderState::Start => {
                        if self.buffer[0] == 70 {
                            if self.buffer[..FILE.len()] != *FILE {
                                return Ok(self.abort());
                            }
                            self.fields.set_type(SequenceType::File);
                            if position == FILE_PART.len() {
                                if self.buffer[FILE.len()..FILE_PART.len()]
                                    != FILE_PART[FILE.len()..]
                                {
                                    return Ok(self.abort());
                                }
                                self.fields.set_type(SequenceType::FilePart);
                                self.state = HeaderState::End;
                                return Ok(ParseOutcome::Payload(index + 1));
                            }
                        } else if self.buffer[0] == 77 {
                            if self.buffer[..MULTIPART.len()] != *MULTIPART {
                                return Ok(self.abort());
                            }
                            self.fields.set_type(SequenceType::MultipartFile);
                        } else {
                            return Ok(self.abort());
                        }
                        state = HeaderState::Key;
                        position = 0;
                    }
                    HeaderState::Key => {
                        if !self.store_key(position) {
                            return Ok(self.abort());
                        }
                        state = HeaderState::Value;
                        position = 0;
                    }
                    HeaderState::Value => {
                        if position >= MAX_FIELD_CHARS {
                            return Ok(self.abort());
                        }
                        self.buffer[position] = code;
                        position += 1;
                    }
                    _ => (),
                },
                58 => {
                    if state == HeaderState::Value && !self.store_value(position) {
                        return Ok(self.abort());
                    }
                    self.state = HeaderState::End;
                    return Ok(ParseOutcome::Payload(index + 1));
                }
                _ => {
                    if position >= MAX_FIELD_CHARS {
                        return Ok(self.abort());
                    }
                    self.buffer[position] = code;
                    position += 1;
                }
            }
        }
        self.state = state;
        self.position = position;
        Ok(ParseOutcome::NeedMore)
    }
    fn abort(&mut self) -> ParseOutcome {
        self.fields.set_type(SequenceType::Invalid);
        self.state = HeaderState::Abort;
        ParseOutcome::Rejected
    }
    fn store_key(&mut self, position: usize) -> bool {
        let key: Vec<_> = self.buffer[..position]
            .iter()
            .map(|&code| (code & 0xffff) as u16)
            .collect();
        if key.is_empty() {
            return false;
        }
        self.key = key;
        self.fields.set(self.key.clone(), HeaderValue::Null);
        true
    }
    fn store_value(&mut self, position: usize) -> bool {
        if self.key.is_empty() {
            return false;
        }
        let Some(value) = decode(&self.key, &self.buffer[..position], self.policy) else {
            return false;
        };
        self.fields.set(self.key.clone(), value);
        true
    }
}
