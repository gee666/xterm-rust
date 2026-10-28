// Copyright (c) 2026 The xterm.js authors. MIT license.
// This independent field inventory intentionally does not call a production
// serializer, so a wrong parser field assignment cannot validate itself.
use serde_json::{json, Map, Value};
use xterm_addons::image::kitty::command::KittyCommand;
use xterm_addons::image::kitty::handler::DecoderSnapshot;

pub fn number(value: f64) -> Value {
    json!(format!("{:016x}", value.to_bits()))
}
pub fn optional(value: Option<f64>) -> Value {
    value.map_or(Value::Null, number)
}
pub fn command(value: &KittyCommand) -> Value {
    let mut map = Map::new();
    macro_rules! n { ($field:ident,$name:literal) => { if let Some(value) = value.$field { map.insert($name.into(), json!({"n":number(value)})); } }; }
    macro_rules! s { ($field:ident,$name:literal) => { if let Some(value) = &value.$field { map.insert($name.into(), json!({"s":value})); } }; }
    s!(action, "action");
    s!(compression, "compression");
    s!(transmission, "transmission");
    s!(delete_selector, "deleteSelector");
    n!(format, "format");
    n!(id, "id");
    n!(image_number, "imageNumber");
    n!(width, "width");
    n!(height, "height");
    n!(x, "x");
    n!(y, "y");
    n!(source_width, "sourceWidth");
    n!(source_height, "sourceHeight");
    n!(x_offset, "xOffset");
    n!(y_offset, "yOffset");
    n!(columns, "columns");
    n!(rows, "rows");
    n!(more, "more");
    n!(quiet, "quiet");
    n!(cursor_movement, "cursorMovement");
    n!(z_index, "zIndex");
    n!(placement_id, "placementId");
    Value::Object(map)
}
pub fn decoder(value: DecoderSnapshot) -> Value {
    json!({"loaded":value.loaded_bytes,"free":value.free_bytes.to_string(),"data":value.data})
}
