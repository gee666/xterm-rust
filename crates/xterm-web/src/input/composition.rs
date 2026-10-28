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
        self.begin_composition();
        let start = textarea.selection_start.unwrap_or(textarea.value.len());
        let end = textarea.selection_end.unwrap_or(start);
        self.set_start_range(start, end);
        self.finish_composition_start(&textarea.value);
    }
    pub fn begin_composition(&mut self) {
        self.is_composing = true;
    }
    pub fn set_start_range(&mut self, start: usize, end: usize) {
        self.start = start.min(end);
        self.end = start.max(end);
    }
    pub fn finish_composition_start(&mut self, value: &[u16]) {
        self.suffix = value[self.end.min(value.len())..].to_vec();
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
        if self.has_pending_composition() {
            if matches!(key_code, 20 | 229 | 16 | 17 | 18) {
                return result;
            }
            self.cancel_for_immediate_send();
            result.data = Some(self.immediate_text(&textarea.value));
        }
        if key_code == 229 {
            if !self.is_textarea_change_pending() {
                result.deferred = Some(self.defer_textarea_change(textarea.value.clone()));
            }
        } else {
            result.continue_processing = true;
        }
        result
    }

    pub fn has_pending_composition(&self) -> bool {
        self.is_composing || self.is_sending
    }

    /// The browser adapter commits these flags before reading live text or
    /// invoking data callbacks. No borrow may span those host operations.
    pub fn cancel_for_immediate_send(&mut self) {
        self.is_composing = false;
        self.is_sending = false;
    }
    pub fn immediate_text(&self, value: &[u16]) -> Vec<u16> {
        substring(value, self.start, self.end).to_vec()
    }
    pub fn is_textarea_change_pending(&self) -> bool {
        self.textarea_change_pending
    }

    /// Check pending and sample old text before this call. The source does not
    /// recheck after that getter, which may itself have reentered input.
    pub fn defer_textarea_change(&mut self, old_value: Vec<u16>) -> Deferred {
        self.textarea_change_scheduled();
        Deferred::TextareaChange { old_value }
    }
    pub fn textarea_change_scheduled(&mut self) {
        self.textarea_change_pending = true;
    }

    pub fn update_end(&mut self, end: usize) {
        self.end = self.start.max(end);
    }

    /// Cancel the pending-send flag before the adapter reads textarea.value.
    pub fn begin_deferred_send(&mut self, start: usize) -> Option<usize> {
        if !self.is_sending {
            return None;
        }
        self.is_sending = false;
        Some(start.saturating_add(self.data_already_sent.len()))
    }
    pub fn text_for_send(
        &self,
        start: usize,
        suffix: &[u16],
        value: &[u16],
        composing: bool,
    ) -> Option<Vec<u16>> {
        let end = if composing {
            self.start
        } else if !suffix.is_empty() && value.ends_with(suffix) {
            start.max(value.len() - suffix.len())
        } else {
            start.max(value.len())
        };
        let input = substring(value, start, end);
        (!input.is_empty()).then(|| input.to_vec())
    }
    pub fn begin_textarea_change(&mut self) -> bool {
        self.textarea_change_pending = false;
        !self.is_composing
    }
    pub fn textarea_diff(&mut self, old_value: &[u16], new_value: &[u16]) -> Option<Vec<u16>> {
        let found = if old_value.is_empty() {
            Some(0)
        } else {
            new_value
                .windows(old_value.len())
                .position(|window| window == old_value)
        };
        let mut diff = new_value.to_vec();
        if let Some(offset) = found {
            diff.drain(offset..offset + old_value.len());
        }
        self.data_already_sent.clone_from(&diff);
        match new_value.len().cmp(&old_value.len()) {
            std::cmp::Ordering::Greater => Some(diff),
            std::cmp::Ordering::Less => Some(vec![0x7f]),
            std::cmp::Ordering::Equal if new_value != old_value => Some(new_value.to_vec()),
            _ => None,
        }
    }

    /// Run with the live textarea snapshot, not the event-time value. DOM hosts
    /// use the same phases separately to avoid early or unnecessary JS getters.
    pub fn run_deferred(&mut self, action: Deferred, textarea: &Textarea) -> Option<Vec<u16>> {
        match action {
            Deferred::UpdateEnd => {
                self.update_end(textarea.selection_end.unwrap_or(textarea.value.len()));
                None
            }
            Deferred::Send { start, suffix } => {
                let start = self.begin_deferred_send(start)?;
                self.text_for_send(start, &suffix, &textarea.value, self.is_composing)
            }
            Deferred::TextareaChange { old_value } => {
                if !self.begin_textarea_change() {
                    return None;
                }
                self.textarea_diff(&old_value, &textarea.value)
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
