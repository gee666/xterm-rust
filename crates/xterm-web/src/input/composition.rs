// Copyright (c) 2016 The xterm.js authors. All rights reserved.
// Licensed under MIT. Ported from browser/input/CompositionHelper.ts.

/// A snapshot sampled by the adapter at event or deferred-callback time.
#[derive(Clone, Debug, Default)]
pub struct Textarea {
    pub value: Vec<u16>,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
}

/// Schedule these actions on the browser's next task turn, in FIFO order.
/// The adapter must discard callbacks when its component is disposed.
#[derive(Clone, Debug)]
pub enum Deferred {
    UpdateEnd,
    Send { start: usize, suffix: Vec<u16> },
    TextareaChange { old_value: Vec<u16> },
}

#[derive(Clone, Debug, Default)]
pub struct KeydownResult {
    pub continue_processing: bool,
    pub data: Option<Vec<u16>>,
    pub deferred: Option<Deferred>,
}

#[derive(Clone, Debug, Default)]
pub struct Composition {
    pub is_composing: bool,
    pub overlay_text: Vec<u16>,
    start: usize,
    end: usize,
    suffix: Vec<u16>,
    is_sending: bool,
    data_already_sent: Vec<u16>,
    textarea_change_pending: bool,
}

impl Composition {
    pub fn start(&mut self, textarea: &Textarea) {
        self.is_composing = true;
        let start = textarea.selection_start.unwrap_or(textarea.value.len());
        let end = textarea.selection_end.unwrap_or(start);
        self.start = start.min(end);
        self.end = start.max(end);
        self.suffix = textarea.value[self.end.min(textarea.value.len())..].to_vec();
        self.overlay_text.clear();
        self.data_already_sent.clear();
    }

    pub fn update(&mut self, data: &[u16]) -> Deferred {
        self.overlay_text = vec![0x200e];
        self.overlay_text.extend(data);
        self.overlay_text.push(0x200e);
        Deferred::UpdateEnd
    }

    pub fn end(&mut self) -> Deferred {
        self.is_composing = false;
        self.is_sending = true;
        Deferred::Send {
            start: self.start,
            suffix: self.suffix.clone(),
        }
    }

    pub fn keydown(&mut self, key_code: u32, textarea: &Textarea) -> KeydownResult {
        let mut result = KeydownResult::default();
        if self.is_composing || self.is_sending {
            if matches!(key_code, 20 | 229 | 16 | 17 | 18) {
                return result;
            }
            self.is_composing = false;
            self.is_sending = false;
            result.data = Some(substring(&textarea.value, self.start, self.end).to_vec());
        }
        if key_code == 229 {
            if !self.textarea_change_pending {
                self.textarea_change_pending = true;
                result.deferred = Some(Deferred::TextareaChange {
                    old_value: textarea.value.clone(),
                });
            }
        } else {
            result.continue_processing = true;
        }
        result
    }

    /// Run with the live textarea snapshot, not the event-time value.
    pub fn run_deferred(&mut self, action: Deferred, textarea: &Textarea) -> Option<Vec<u16>> {
        match action {
            Deferred::UpdateEnd => {
                self.end = self
                    .start
                    .max(textarea.selection_end.unwrap_or(textarea.value.len()));
                None
            }
            Deferred::Send { start, suffix } => {
                if !self.is_sending {
                    return None;
                }
                self.is_sending = false;
                let start = start.saturating_add(self.data_already_sent.len());
                let end = if self.is_composing {
                    self.start
                } else if !suffix.is_empty() && textarea.value.ends_with(&suffix) {
                    start.max(textarea.value.len() - suffix.len())
                } else {
                    start.max(textarea.value.len())
                };
                let input = substring(&textarea.value, start, end);
                (!input.is_empty()).then(|| input.to_vec())
            }
            Deferred::TextareaChange { old_value } => {
                self.textarea_change_pending = false;
                if self.is_composing {
                    return None;
                }
                let new_value = &textarea.value;
                let found = if old_value.is_empty() {
                    Some(0)
                } else {
                    new_value
                        .windows(old_value.len())
                        .position(|window| window == old_value)
                };
                let mut diff = new_value.clone();
                if let Some(offset) = found {
                    diff.drain(offset..offset + old_value.len());
                }
                self.data_already_sent.clone_from(&diff);
                match new_value.len().cmp(&old_value.len()) {
                    std::cmp::Ordering::Greater => Some(diff),
                    std::cmp::Ordering::Less => Some(vec![0x7f]),
                    std::cmp::Ordering::Equal if new_value != &old_value => Some(new_value.clone()),
                    _ => None,
                }
            }
        }
    }
}

// JS substring clamps both endpoints, then swaps them if reversed.
fn substring(value: &[u16], start: usize, end: usize) -> &[u16] {
    let start = start.min(value.len());
    let end = end.min(value.len());
    &value[start.min(end)..start.max(end)]
}

#[cfg(test)]
#[path = "composition_tests.rs"]
mod tests;
