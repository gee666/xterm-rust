// Licensed under MIT. Shared protocol for browser-input fixtures and JS oracle.
use std::collections::VecDeque;
use std::error::Error;
use xterm_web::input::{
    clipboard,
    composition::{Composition, Deferred, Textarea},
};

#[derive(Default)]
pub struct InputTrace {
    composition: Composition,
    textarea: Textarea,
    deferred: VecDeque<Deferred>,
    data: Vec<Vec<u16>>,
}

fn units(text: &str) -> Result<Vec<u16>, std::num::ParseIntError> {
    if text == "~" {
        return Ok(Vec::new());
    }
    text.split(',')
        .filter(|part| !part.is_empty())
        .map(|part| u16::from_str_radix(part, 16))
        .collect()
}

fn hex(units: &[u16]) -> String {
    if units.is_empty() {
        "~".into()
    } else {
        units
            .iter()
            .map(|unit| format!("{unit:x}"))
            .collect::<Vec<_>>()
            .join(",")
    }
}

impl InputTrace {
    pub fn apply(&mut self, line: &str) -> Result<String, Box<dyn Error>> {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() < 4 {
            return Err("expected four input fields".into());
        }
        let mut continuing = "-";
        match fields[0] {
            "prepare" => {
                return Ok(hex(&clipboard::prepare_text_for_terminal(&units(
                    fields[1],
                )?)))
            }
            "bracket" => {
                return Ok(hex(&clipboard::bracket_text_for_paste(
                    &units(fields[1])?,
                    fields[2] == "1",
                )))
            }
            "paste" => {
                return Ok(hex(&clipboard::paste(
                    &units(fields[1])?,
                    fields[2] == "1",
                    fields[3] == "1",
                )))
            }
            "reset" => *self = Self::default(),
            "value" => self.textarea.value = units(fields[1])?,
            "startsel" => {
                self.textarea.selection_start = if fields[2] == "-" {
                    None
                } else {
                    Some(fields[2].parse()?)
                }
            }
            "endsel" => {
                self.textarea.selection_end = if fields[2] == "-" {
                    None
                } else {
                    Some(fields[2].parse()?)
                }
            }
            "start" => self.composition.start(&self.textarea),
            "update" => self
                .deferred
                .push_back(self.composition.update(&units(fields[1])?)),
            "end" => self.deferred.push_back(self.composition.end()),
            "key" => {
                let result = self.composition.keydown(fields[2].parse()?, &self.textarea);
                continuing = if result.continue_processing { "1" } else { "0" };
                if let Some(data) = result.data {
                    self.data.push(data);
                }
                if let Some(action) = result.deferred {
                    self.deferred.push_back(action);
                }
            }
            "tick" => {
                let action = self.deferred.pop_front().ok_or("no pending callback")?;
                if let Some(data) = self.composition.run_deferred(action, &self.textarea) {
                    self.data.push(data);
                }
            }
            _ => return Err("unknown browser input operation".into()),
        }
        let emitted = if self.data.is_empty() {
            "-".into()
        } else {
            self.data
                .iter()
                .map(|data| hex(data))
                .collect::<Vec<_>>()
                .join("/")
        };
        Ok(format!(
            "{};{continuing};{};{emitted};{}",
            u8::from(self.composition.is_composing),
            hex(&self.composition.overlay_text),
            self.deferred.len()
        ))
    }
}
