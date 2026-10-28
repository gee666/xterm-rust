//! One source-ordered rule generator for native and live UTF-16 CSS readers.
#[derive(Clone, Copy)]
pub(crate) enum ColorField {
    Foreground,
    ForegroundHalf,
    BackgroundOpaque,
    BackgroundOpaqueHalf,
    Cursor,
    CursorAccent,
    Selection,
    InactiveSelection,
    Ansi(usize),
    AnsiHalf(usize),
}
#[derive(Clone, Copy)]
pub(crate) enum OptionField {
    Family,
    Size,
    Weight,
    WeightBold,
    CursorWidth,
}
impl OptionField {
    pub fn name(self) -> &'static str {
        match self {
            Self::Family => "fontFamily",
            Self::Size => "fontSize",
            Self::Weight => "fontWeight",
            Self::WeightBold => "fontWeightBold",
            Self::CursorWidth => "cursorWidth",
        }
    }
}
#[derive(Clone, Copy)]
pub(crate) enum Field {
    Color(ColorField),
    Option(OptionField),
}
pub(crate) trait CssReader {
    type Error;
    fn read(&self, field: Field) -> Result<Vec<u16>, Self::Error>;
    fn palette_len(&self) -> usize;
}
#[derive(Default)]
struct Output(Vec<u16>);
impl Output {
    fn text(&mut self, text: &str) {
        self.0.extend(text.encode_utf16());
    }
    fn open(&mut self, selector: &str) {
        self.text(selector);
        self.text(" {");
    }
    fn close(&mut self) {
        self.text("}");
    }
    fn value<S: CssReader>(
        &mut self,
        source: &S,
        prefix: &str,
        field: Field,
        suffix: &str,
    ) -> Result<(), S::Error> {
        self.text(prefix);
        self.0.extend(source.read(field)?);
        self.text(suffix);
        Ok(())
    }
    fn rule(&mut self, selector: &str, declarations: &str) {
        self.open(selector);
        self.text(declarations);
        self.close();
    }
    fn rule_value<S: CssReader>(
        &mut self,
        source: &S,
        selector: &str,
        prefix: &str,
        field: Field,
        suffix: &str,
    ) -> Result<(), S::Error> {
        self.open(selector);
        self.value(source, prefix, field, suffix)?;
        self.close();
        Ok(())
    }
}

pub(crate) fn build<S: CssReader>(id: u64, source: &S) -> Result<Vec<u16>, S::Error> {
    use ColorField::*;
    use Field::{Color as C, Option as O};
    use OptionField::*;
    const INVERTED_DEFAULT_COLOR: usize = 257;
    let selector = format!(".xterm-dom-renderer-owner-{id}");
    let rows = format!("{selector} .xterm-rows");
    let cursor = format!("{rows} .xterm-cursor");
    let mut out = Output::default();
    out.rule_value(
        source,
        &rows,
        " pointer-events: none; color: ",
        C(Foreground),
        ";",
    )?;
    out.open(&format!("{rows}, {rows} span"));
    out.value(source, " font-family: ", O(Family), ";")?;
    out.value(
        source,
        " font-size: ",
        O(Size),
        "px; font-kerning: none; white-space: pre",
    )?;
    out.close();
    out.rule_value(
        source,
        &format!("{rows} .xterm-dim"),
        " color: ",
        C(ForegroundHalf),
        ";",
    )?;
    out.rule_value(
        source,
        &format!("{selector} span:not(.xterm-bold)"),
        " font-weight: ",
        O(Weight),
        ";",
    )?;
    out.rule_value(
        source,
        &format!("{selector} span.xterm-bold"),
        " font-weight: ",
        O(WeightBold),
        ";",
    )?;
    out.rule(
        &format!("{selector} span.xterm-italic"),
        " font-style: italic;",
    );
    out.rule(
        &format!("{selector} span.xterm-blink-hidden"),
        " visibility: hidden;",
    );
    let underline = format!("blink_underline_{id}");
    let bar = format!("blink_bar_{id}");
    let block = format!("blink_block_{id}");
    out.text(&format!(
        "@keyframes {underline} {{ 50% {{  border-bottom-style: hidden; }}}}"
    ));
    out.text(&format!(
        "@keyframes {bar} {{ 50% {{  box-shadow: none; }}}}"
    ));
    out.text(&format!("@keyframes {block} {{ 0% {{"));
    out.value(source, "  background-color: ", C(Cursor), ";")?;
    out.value(
        source,
        "  color: ",
        C(CursorAccent),
        "; } 50% {  background-color: inherit;",
    )?;
    out.value(source, "  color: ", C(Cursor), "; }}")?;
    let focused = format!("{rows}.xterm-focus .xterm-cursor.xterm-cursor-blink");
    for (style, animation) in [("underline", underline), ("bar", bar), ("block", block)] {
        out.rule(
            &format!("{focused}.xterm-cursor-{style}"),
            &format!(" animation: {animation} 1s step-end infinite;"),
        );
    }
    out.rule(
        &format!("{rows}.xterm-cursor-blink-idle .xterm-cursor.xterm-cursor-blink"),
        " animation: none !important;",
    );
    out.open(&format!("{cursor}.xterm-cursor-block"));
    out.value(source, " background-color: ", C(Cursor), ";")?;
    out.value(source, " color: ", C(CursorAccent), ";")?;
    out.close();
    out.open(&format!(
        "{cursor}.xterm-cursor-block:not(.xterm-cursor-blink)"
    ));
    out.value(source, " background-color: ", C(Cursor), " !important;")?;
    out.value(source, " color: ", C(CursorAccent), " !important;")?;
    out.close();
    out.rule_value(
        source,
        &format!("{cursor}.xterm-cursor-outline"),
        " outline: 1px solid ",
        C(Cursor),
        "; outline-offset: -1px;",
    )?;
    out.open(&format!("{cursor}.xterm-cursor-bar"));
    out.value(source, " box-shadow: ", O(CursorWidth), "px 0 0 ")?;
    out.value(source, "", C(Cursor), " inset;")?;
    out.close();
    out.rule_value(
        source,
        &format!("{cursor}.xterm-cursor-underline"),
        " border-bottom: 1px ",
        C(Cursor),
        "; border-bottom-style: solid; height: calc(100% - 1px);",
    )?;
    out.rule(
        &format!("{selector} .xterm-selection"),
        " position: absolute; top: 0; left: 0; z-index: 1; pointer-events: none;",
    );
    out.rule_value(
        source,
        &format!("{selector}.focus .xterm-selection div"),
        " position: absolute; background-color: ",
        C(Selection),
        ";",
    )?;
    out.rule_value(
        source,
        &format!("{selector} .xterm-selection div"),
        " position: absolute; background-color: ",
        C(InactiveSelection),
        ";",
    )?;
    for index in 0..source.palette_len() {
        out.rule_value(
            source,
            &format!("{selector} .xterm-fg-{index}"),
            " color: ",
            C(Ansi(index)),
            "; ",
        )?;
        out.rule_value(
            source,
            &format!("{selector} .xterm-fg-{index}.xterm-dim"),
            " color: ",
            C(AnsiHalf(index)),
            "; ",
        )?;
        out.rule_value(
            source,
            &format!("{selector} .xterm-bg-{index}"),
            " background-color: ",
            C(Ansi(index)),
            "; ",
        )?;
    }
    out.rule_value(
        source,
        &format!("{selector} .xterm-fg-{INVERTED_DEFAULT_COLOR}"),
        " color: ",
        C(BackgroundOpaque),
        "; ",
    )?;
    out.rule_value(
        source,
        &format!("{selector} .xterm-fg-{INVERTED_DEFAULT_COLOR}.xterm-dim"),
        " color: ",
        C(BackgroundOpaqueHalf),
        "; ",
    )?;
    out.rule_value(
        source,
        &format!("{selector} .xterm-bg-{INVERTED_DEFAULT_COLOR}"),
        " background-color: ",
        C(Foreground),
        "; ",
    )?;
    Ok(out.0)
}
