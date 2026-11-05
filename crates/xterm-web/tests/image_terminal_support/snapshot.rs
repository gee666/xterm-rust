// Copyright (c) 2020 The xterm.js authors. MIT license.
use wasm_bindgen::JsValue;
use xterm_addons::image::storage::ImageTileInfo;
use xterm_web::Terminal;
pub fn snapshot(terminal: &Terminal) -> JsValue {
    let columns = terminal.shared_native().cols().unwrap();
    terminal
        .shared_native()
        .with_state(|state| {
            let buffer = state.active_buffer();
            let buffer = buffer.borrow();
            let rows = js_sys::Array::new();
            for y in 0..buffer.lines.len() {
                let line = buffer.lines.get(y).unwrap();
                let line = line.borrow();
                let cells = js_sys::Array::new();
                for x in 0..columns {
                    let image = line
                        .get_extended(x)
                        .and_then(|e| e.payload())
                        .and_then(|p| p.downcast::<ImageTileInfo>().ok());
                    cells.push(&js_sys::Array::of5(
                        &js_sys::JsString::from_char_code(&line.get_string(x).unwrap()).into(),
                        &line.get_width(x).unwrap().into(),
                        &line.get_fg(x).unwrap().into(),
                        &line.get_bg(x).unwrap().into(),
                        &image.map_or(JsValue::NULL, |p| {
                            js_sys::Array::of2(
                                &(p.image_id.get() as f64).into(),
                                &p.tile_id.get().into(),
                            )
                            .into()
                        }),
                    ));
                }
                rows.push(&js_sys::Array::of2(&line.is_wrapped.into(), &cells));
            }
            js_sys::Array::of5(
                &(buffer.x as f64).into(),
                &(buffer.y as f64).into(),
                &(buffer.ybase as f64).into(),
                &(buffer.ydisp as f64).into(),
                &rows,
            )
            .into()
        })
        .unwrap()
}
