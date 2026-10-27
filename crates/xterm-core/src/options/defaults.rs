// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Adapted from common/services/OptionsService.ts.
use super::OptionValue as V;

// Like the shallow copies of DEFAULT_OPTIONS, object defaults share identity
// across services on the same terminal/UI thread.
thread_local! {
    static DEFAULTS: Vec<(&'static str, V)> = build();
}

pub fn default_options(is_mac: bool) -> Vec<(&'static str, V)> {
    DEFAULTS.with(|defaults| {
        defaults
            .iter()
            .map(|(key, value)| {
                (
                    *key,
                    if *key == "rightClickSelectsWord" {
                        V::Bool(is_mac)
                    } else {
                        value.clone()
                    },
                )
            })
            .collect()
    })
}

fn build() -> Vec<(&'static str, V)> {
    vec![
        ("cols", V::Number(80.0)),
        ("rows", V::Number(24.0)),
        ("showCursorImmediately", V::Bool(false)),
        ("cursorBlink", V::Bool(false)),
        ("blinkIntervalDuration", V::Number(0.0)),
        ("cursorStyle", V::from("block")),
        ("cursorWidth", V::Number(1.0)),
        ("cursorInactiveStyle", V::from("outline")),
        ("drawBoldTextInBrightColors", V::Bool(true)),
        ("documentOverride", V::Null),
        ("fastScrollSensitivity", V::Number(5.0)),
        ("fontFamily", V::from("monospace")),
        ("fontSize", V::Number(15.0)),
        ("fontWeight", V::from("normal")),
        ("fontWeightBold", V::from("bold")),
        ("ignoreBracketedPasteMode", V::Bool(false)),
        ("lineHeight", V::Number(1.0)),
        ("letterSpacing", V::Number(0.0)),
        ("linkHandler", V::Null),
        ("logLevel", V::from("info")),
        ("logger", V::Null),
        ("scrollback", V::Number(1000.0)),
        (
            "scrollbar",
            V::property_map([("showScrollbar", V::Bool(true))]),
        ),
        ("scrollOnEraseInDisplay", V::Bool(false)),
        ("scrollOnUserInput", V::Bool(true)),
        ("scrollSensitivity", V::Number(1.0)),
        ("screenReaderMode", V::Bool(false)),
        ("smoothScrollDuration", V::Number(0.0)),
        ("macOptionIsMeta", V::Bool(false)),
        ("macOptionClickForcesSelection", V::Bool(false)),
        ("minimumContrastRatio", V::Number(1.0)),
        ("mouseEventsRequireAlt", V::Bool(false)),
        ("disableStdin", V::Bool(false)),
        ("allowProposedApi", V::Bool(false)),
        ("allowTransparency", V::Bool(false)),
        ("tabStopWidth", V::Number(8.0)),
        ("theme", V::object(())),
        ("reflowCursorLine", V::Bool(false)),
        ("rescaleOverlappingGlyphs", V::Bool(false)),
        ("rightClickSelectsWord", V::Bool(false)),
        ("windowOptions", V::object(())),
        ("windowsPty", V::from(super::WindowsPty::default())),
        ("wordSeparator", V::from(" ()[]{}',\"`")),
        ("altClickMovesCursor", V::Bool(true)),
        ("convertEol", V::Bool(false)),
        ("termName", V::from("xterm")),
        ("quirks", V::object(())),
        ("vtExtensions", V::object(())),
    ]
}
