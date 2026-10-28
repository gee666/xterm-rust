// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! The scrollable element's five-sample physical-wheel classifier.
use super::math::{max, min, round};
use std::collections::VecDeque;
use xterm_core::options::{OptionError, OptionValue as V};

const CAPACITY: usize = 5;
const INTEGER_TOLERANCE: f64 = 0.01;
#[derive(Clone, Debug)]
struct Item {
    x: V,
    y: V,
    score: f64,
}
#[derive(Clone, Debug, Default)]
pub struct MouseWheelClassifier {
    memory: VecDeque<Item>,
}
impl MouseWheelClassifier {
    /// Upstream stores but never reads timestamps. Keep the clock argument for
    /// trace compatibility without retaining unused per-sample data.
    pub fn accept(&mut self, timestamp: f64, x: f64, y: f64) -> Result<(), OptionError> {
        self.accept_values(timestamp, V::Number(x), V::Number(y))
    }
    /// Normalization produces arithmetic primitives. Retaining those values
    /// matters for BigInt failures: source inserts a score-zero sample before
    /// Math.abs/round can throw, so the next valid sample sees that history.
    pub(crate) fn accept_values(&mut self, _timestamp: f64, x: V, y: V) -> Result<(), OptionError> {
        let previous = self.memory.back().cloned();
        if self.memory.len() == CAPACITY {
            self.memory.pop_front();
        }
        self.memory.push_back(Item {
            x: x.clone(),
            y: y.clone(),
            score: 0.0,
        });
        let score = score(&x, &y, previous.as_ref())?;
        // No user event is emitted while computing a normalized primitive's
        // score; insertion above guarantees a current back entry.
        if let Some(item) = self.memory.back_mut() {
            item.score = score;
        }
        Ok(())
    }
    pub fn is_physical_mouse_wheel(&self) -> bool {
        if self.memory.is_empty() {
            return false;
        }
        let mut remaining = 1.0;
        let mut score = 0.0;
        for (index, item) in self.memory.iter().rev().enumerate() {
            let influence = if index + 1 == self.memory.len() {
                remaining
            } else {
                2.0_f64.powi(-(index as i32 + 1))
            };
            remaining -= influence;
            score += item.score * influence;
        }
        score <= 0.5
    }
}
fn abs(value: &V) -> Result<f64, OptionError> {
    Ok(value.to_number()?.abs())
}
fn score(x: &V, y: &V, previous: Option<&Item>) -> Result<f64, OptionError> {
    if abs(x)? > 0.0 && abs(y)? > 0.0 {
        return Ok(1.0);
    }
    let mut score = 0.5;
    if !almost_int(x)? || !almost_int(y)? {
        score += 0.25;
    }
    if let Some(previous) = previous {
        let x = abs(x)?;
        let y = abs(y)?;
        let previous_x = abs(&previous.x)?;
        let previous_y = abs(&previous.y)?;
        let min_x = max(min(x, previous_x), 1.0);
        let min_y = max(min(y, previous_y), 1.0);
        let max_x = max(x, previous_x);
        let max_y = max(y, previous_y);
        if max_x % min_x == 0.0 && max_y % min_y == 0.0 {
            score -= 0.5;
        }
    }
    Ok(min(max(score, 0.0), 1.0))
}
fn almost_int(value: &V) -> Result<bool, OptionError> {
    Ok((round(value.to_number()?) - value.to_number()?).abs() < INTEGER_TOLERANCE)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_integer_repeated_steps_and_diagonal_trackpad() {
        let mut classifier = MouseWheelClassifier::default();
        assert!(!classifier.is_physical_mouse_wheel());
        classifier.accept(1.0, 0.0, 1.0).unwrap();
        assert!(classifier.is_physical_mouse_wheel());
        classifier.accept(2.0, 0.0, 2.0).unwrap();
        assert!(classifier.is_physical_mouse_wheel());
        for timestamp in 3..10 {
            classifier.accept(f64::from(timestamp), 0.5, 0.2).unwrap();
        }
        assert!(!classifier.is_physical_mouse_wheel());
        assert_eq!(classifier.memory.len(), CAPACITY);
    }
    #[test]
    fn fractional_and_nonfinite_samples_are_not_integer_wheels() {
        for value in [0.5, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut classifier = MouseWheelClassifier::default();
            classifier.accept(0.0, 0.0, value).unwrap();
            assert!(!classifier.is_physical_mouse_wheel());
        }
    }
}
