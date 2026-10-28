use serde_json::{json, Value};
use std::error::Error;
use xterm_addons::image::header::{HeaderParser, HeaderValue};
use xterm_addons::image::metrics::image_type;
use xterm_addons::image::NameDecodePolicy;

fn field(value: &HeaderValue) -> Value {
    match value {
        HeaderValue::Number(number) => json!(["n", format!("{:016x}", number.to_bits())]),
        HeaderValue::Text(units) => json!(["s", units]),
        HeaderValue::Codepoints(units) => json!(["a", units]),
        HeaderValue::Null => json!(["null"]),
        HeaderValue::Undefined => json!(["u"]),
        HeaderValue::Boolean(value) => json!(["b", value]),
        HeaderValue::DecoderObject => json!(["o"]),
    }
}
pub fn snapshot(parser: &HeaderParser, result: i64) -> Value {
    let end = parser
        .buffer()
        .iter()
        .rposition(|unit| *unit != 0)
        .map_or(0, |index| index + 1);
    json!({"result":result,"state":parser.state as u8,"position":parser.position(),"key":parser.current_key(),"buffer":parser.buffer()[..end],"nullPrototype":parser.fields.has_null_prototype(),"fields":parser.fields.entries().iter().map(|(key,value)|json!([key,field(value)])).collect::<Vec<_>>()})
}
pub fn apply(parser: &mut HeaderParser, operation: &Value) -> Result<Value, Box<dyn Error>> {
    let command = operation["op"].as_str().ok_or("missing op")?;
    let result = match command {
        "new" => {
            let policy = if operation["policy"] == "browser" {
                NameDecodePolicy::BrowserAtob
            } else {
                NameDecodePolicy::NodeBuffer
            };
            *parser = HeaderParser::new(policy);
            0
        }
        "reset" => {
            parser.reset();
            0
        }
        "end" => parser.end().source_code(),
        "parse" => {
            let data = operation["data"]
                .as_array()
                .ok_or("missing data")?
                .iter()
                .map(|unit| {
                    unit.as_u64()
                        .ok_or("bad codepoint")
                        .and_then(|unit| u32::try_from(unit).map_err(|_| "large codepoint"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let start = usize::try_from(operation["start"].as_u64().ok_or("missing start")?)?;
            let end = usize::try_from(operation["end"].as_u64().ok_or("missing end")?)?;
            parser.parse(&data, start, end)?.source_code()
        }
        "metrics" => {
            let file_data;
            let encoded_data;
            let input = if let Some(file) = operation["file"].as_str() {
                if std::path::Path::new(file)
                    .file_name()
                    .and_then(|s| s.to_str())
                    != Some(file)
                {
                    return Err("invalid image fixture basename".into());
                }
                let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../source_repo/addons/addon-image/fixture/testimages");
                file_data = std::fs::read(root.join(file))?;
                encoded_data = json!(file_data);
                &encoded_data
            } else {
                &operation["data"]
            };
            let data = input
                .as_array()
                .ok_or("missing data")?
                .iter()
                .map(|byte| {
                    byte.as_u64()
                        .ok_or("bad byte")
                        .and_then(|byte| u8::try_from(byte).map_err(|_| "large byte"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let result = image_type(&data);
            return Ok(
                json!({"mime":result.mime.mime(),"width":result.width,"height":result.height}),
            );
        }
        _ => return Err("unknown image helper operation".into()),
    };
    Ok(snapshot(parser, result))
}
