// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Lossless, source-ordered CSS without pre-coercing live option objects.
use super::{
    theme_css::{
        color_units,
        rules::{self, CssReader, Field},
        snapshot_units,
    },
    width_cache::Font,
};
use crate::theme::{ColorSet, SharedColors};
use xterm_core::options::{OptionError, OptionsService};

/// Snapshot convenience path for raw color strings and well-formed native fonts.
/// The live browser uses theme_css_live to preserve option getter/coercion timing.
pub fn theme_css_utf16(id: u64, colors: &ColorSet, font: &Font, cursor_width: f64) -> Vec<u16> {
    snapshot_units(id, colors, font, cursor_width, true)
}

/// Reads each option at the same interpolation position as DomRenderer._injectCss.
/// In particular, font size uses String hint, not ToNumber, and a family/weight
/// conversion may mutate colors observed by subsequent rules. No shared guard
/// remains held during any option lookup or user-owned primitive conversion.
pub fn theme_css_live(
    id: u64,
    colors: &SharedColors,
    options: &OptionsService,
) -> Result<Vec<u16>, OptionError> {
    rules::build(id, &Live { colors, options })
}
struct Live<'a> {
    colors: &'a SharedColors,
    options: &'a OptionsService,
}
impl CssReader for Live<'_> {
    type Error = OptionError;
    fn palette_len(&self) -> usize {
        self.colors.borrow().ansi.len()
    }
    fn read(&self, field: Field) -> Result<Vec<u16>, OptionError> {
        match field {
            Field::Color(field) => Ok(color_units(&self.colors.borrow(), field, true)),
            Field::Option(field) => self.options.get(field.name())?.to_utf16(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeService;
    use std::{cell::RefCell, rc::Rc};
    use xterm_core::options::{CoercionHint, OptionObject, OptionValue};

    #[test]
    fn raw_color_units_are_not_replaced_or_recursively_decoded() {
        let theme = ThemeService::default();
        let colors = theme.colors();
        let units = vec![
            0x7e, 0x30, 0x30, 0x34, 0x31, 0x7e, 0, 0xd800, 0xdc00, 0xffff,
        ];
        colors.borrow().foreground.set_css_units(units.clone());
        colors.borrow_mut().ansi.clear();
        let output = theme_css_utf16(1, &colors.borrow(), &Font::default(), 1.0);
        assert!(output.windows(units.len()).any(|part| part == units));
    }

    #[test]
    fn font_conversion_mutations_are_visible_only_to_later_color_rules() {
        let theme = ThemeService::default();
        let colors = theme.colors();
        colors.borrow_mut().ansi.clear();
        let hints = Rc::new(RefCell::new(Vec::new()));
        let changed = colors.clone();
        let observed = hints.clone();
        let family = OptionObject::with_properties(
            (),
            move |hint| {
                observed.borrow_mut().push(hint);
                changed
                    .borrow()
                    .foreground
                    .set_css_units("new-foreground".encode_utf16().collect());
                changed.borrow().foreground.borrow_mut().rgba = 0x00ff00ff;
                Ok(OptionValue::from("monospace"))
            },
            |_, _| Ok(OptionValue::Undefined),
        );
        let options = OptionsService::new(false, [] as [(&str, OptionValue); 0]);
        options
            .set_raw("fontFamily", OptionValue::Object(family))
            .unwrap();
        let output = theme_css_live(1, &colors, &options).unwrap();
        let output = String::from_utf16(&output).unwrap();
        assert!(output.contains("pointer-events: none; color: #ffffff;"));
        assert!(output.contains(".xterm-rows .xterm-dim { color: #00ff0080;"));
        assert!(output.contains(".xterm-bg-257 { background-color: new-foreground; }"));
        assert_eq!(*hints.borrow(), [CoercionHint::String]);
    }
}
