// Copyright (c) 2016 The xterm.js authors. All rights reserved. MIT license.
//! Environment detection from supplied metadata, never from core browser globals.
use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct Navigator<'a> {
    pub user_agent: &'a str,
    pub platform: &'a str,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MissingNavigator;
impl fmt::Display for MissingNavigator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("navigator is required outside Node.js")
    }
}
impl std::error::Error for MissingNavigator {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Platform {
    pub is_node: bool,
    pub is_firefox: bool,
    pub is_chrome: bool,
    pub is_legacy_edge: bool,
    pub is_safari: bool,
    pub is_mac: bool,
    pub is_windows: bool,
    pub is_linux: bool,
    pub is_chrome_os: bool,
    pub safari_version: f64,
}
impl Platform {
    /// `process_has_title` corresponds to property presence, not truthiness.
    /// A process polyfill in a browser must not force Node behavior.
    pub fn detect(
        process_has_title: bool,
        navigator: Option<Navigator<'_>>,
    ) -> Result<Self, MissingNavigator> {
        let is_node =
            process_has_title && navigator.is_none_or(|n| n.user_agent.starts_with("Node.js/"));
        let navigator = if is_node {
            Navigator {
                user_agent: "node",
                platform: "node",
            }
        } else {
            navigator.ok_or(MissingNavigator)?
        };
        let ua = navigator.user_agent;
        let platform = navigator.platform;
        let lower = ua.to_ascii_lowercase();
        // The source regex is anchored only at the start, not at the end.
        // Chrome after Safari does not disqualify a match; line breaks do.
        let first_line = lower
            .split(['\n', '\r', '\u{2028}', '\u{2029}'])
            .next()
            .unwrap_or("");
        let is_safari = first_line.find("safari").is_some_and(|index| {
            !first_line[..index].contains("chrome") && !first_line[..index].contains("android")
        });
        let safari_version = if is_safari {
            ua.match_indices("Version/")
                .find_map(|(index, _)| {
                    let digits: String = ua[index + "Version/".len()..]
                        .chars()
                        .take_while(char::is_ascii_digit)
                        .collect();
                    if digits.is_empty() {
                        None
                    } else {
                        digits.parse::<f64>().ok()
                    }
                })
                .unwrap_or(0.0)
        } else {
            0.0
        };
        Ok(Self {
            is_node,
            is_firefox: ua.contains("Firefox"),
            is_chrome: ua.contains("Chrome"),
            is_legacy_edge: ua.contains("Edge"),
            is_safari,
            is_mac: ["Macintosh", "MacIntel", "MacPPC", "Mac68K"].contains(&platform),
            is_windows: ["Windows", "Win16", "Win32", "WinCE"].contains(&platform),
            is_linux: platform.contains("Linux"),
            is_chrome_os: contains_word(ua, "CrOS"),
            safari_version,
        })
    }
}
fn contains_word(text: &str, needle: &str) -> bool {
    let is_word = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
    text.match_indices(needle).any(|(index, _)| {
        (index == 0 || !is_word(text.as_bytes()[index - 1]))
            && (index + needle.len() == text.len()
                || !is_word(text.as_bytes()[index + needle.len()]))
    })
}
/// Upstream intentionally returns one, ignoring device pixel ratio.
pub fn get_zoom_factor(_device_pixel_ratio: Option<f64>) -> f64 {
    1.0
}
