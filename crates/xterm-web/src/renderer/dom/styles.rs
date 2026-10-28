use super::*;
impl DomRenderer {
    pub(super) fn inject_css(&self) -> Result<(), JsValue> {
        let colors = self.theme.colors();
        let units = crate::renderer::theme_css_utf16::theme_css_live(
            self.terminal_id,
            &colors,
            &self.options_service,
        )
        .map_err(|error| crate::js_error::to_js_error(&error))?;
        // Lazy input evaluation is complete and owns its output. No color/font
        // borrow survives this strict host setter, which may throw or reenter.
        crate::utf16::set_text_content(
            self.theme_style.as_ref(),
            crate::utf16::to_js_string(&units),
        )
    }
}
