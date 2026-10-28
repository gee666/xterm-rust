// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use super::commands::DemoResult;
use std::{ffi::OsString, path::PathBuf};

#[derive(Default)]
pub struct Arguments {
    pub script: Option<PathBuf>,
    pub plain: bool,
    pub cell_budget: Option<usize>,
    pub help: bool,
}

pub fn parse(arguments: impl IntoIterator<Item = OsString>) -> DemoResult<Arguments> {
    let mut parsed = Arguments::default();
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        match argument
            .to_str()
            .ok_or("command-line option is not valid UTF-8")?
        {
            "--script" => {
                if parsed.script.is_some() {
                    return Err("--script supplied twice".into());
                }
                parsed.script = Some(PathBuf::from(
                    arguments.next().ok_or("--script needs a file path")?,
                ));
            }
            "--plain" => parsed.plain = true,
            "--max-cells" => {
                if parsed.cell_budget.is_some() {
                    return Err("--max-cells supplied twice".into());
                }
                let value = arguments
                    .next()
                    .ok_or("--max-cells needs a positive integer")?;
                parsed.cell_budget = Some(
                    value
                        .to_str()
                        .ok_or("--max-cells needs UTF-8 digits")?
                        .parse()?,
                );
            }
            "--help" | "-h" => {
                parsed.help = true;
                return Ok(parsed);
            }
            _ => return Err(format!("unknown argument {argument:?}").into()),
        }
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(|value| OsString::from(*value)).collect()
    }
    #[test]
    fn parses_flags_without_changing_script_path() {
        let parsed = parse(args(&[
            "--plain",
            "--script",
            "a path/script.txt",
            "--max-cells",
            "400",
        ]))
        .unwrap();
        assert!(parsed.plain);
        assert_eq!(parsed.script, Some(PathBuf::from("a path/script.txt")));
        assert_eq!(parsed.cell_budget, Some(400));
        assert!(!parsed.help);
        assert!(parse(args(&["--help"])).unwrap().help);
    }
    #[test]
    fn invalid_and_repeated_flags_are_errors() {
        for values in [
            vec!["--unknown"],
            vec!["--script"],
            vec!["--max-cells"],
            vec!["--max-cells", "-1"],
            vec!["--max-cells", "bad"],
            vec!["--script", "one", "--script", "two"],
            vec!["--max-cells", "400", "--max-cells", "500"],
        ] {
            assert!(parse(args(&values)).is_err(), "{values:?}");
        }
    }
    #[cfg(unix)]
    #[test]
    fn non_utf8_options_fail_but_script_paths_remain_lossless() {
        use std::os::unix::ffi::OsStringExt;
        let value = OsString::from_vec(vec![0xff]);
        assert!(parse([value.clone()]).is_err());
        let parsed = parse([OsString::from("--script"), value.clone()]).unwrap();
        assert_eq!(parsed.script.unwrap().as_os_str(), value.as_os_str());
    }
    #[cfg(windows)]
    #[test]
    fn non_unicode_options_fail_but_script_paths_remain_lossless() {
        use std::os::windows::ffi::OsStringExt;
        let value = OsString::from_wide(&[0xd800]);
        assert!(parse([value.clone()]).is_err());
        let parsed = parse([OsString::from("--script"), value.clone()]).unwrap();
        assert_eq!(parsed.script.unwrap().as_os_str(), value.as_os_str());
    }
}
