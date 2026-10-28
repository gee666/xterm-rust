//! Native IIP protocol/geometry replay. A display request is not a rendered image.
use serde_json::{json, Value};
use std::error::Error;
use std::io::{self, BufRead};
use xterm_addons::image::header::{HeaderFields, HeaderValue};
use xterm_addons::image::iip::geometry::{CellSize, IipGeometry, RenderMetrics, SizeRequest};
use xterm_addons::image::iip::handler::IipHandler;
use xterm_addons::image::iip::reports::fixed_three;
use xterm_addons::image::iip::types::{EndPreparation, IipEffect, IipOptions};
use xterm_addons::image::NameDecodePolicy;

fn number(value: &Value) -> Result<f64, Box<dyn Error>> {
    let bits = u64::from_str_radix(value.as_str().ok_or("missing float bits")?, 16)?;
    Ok(f64::from_bits(bits))
}
fn bits(value: f64) -> String {
    if value.is_nan() {
        "NaN".to_owned()
    } else {
        format!("{:016x}", value.to_bits())
    }
}
fn units(value: &Value) -> Result<Vec<u16>, Box<dyn Error>> {
    value
        .as_array()
        .ok_or("missing UTF16 array")?
        .iter()
        .map(|v| Ok(u16::try_from(v.as_u64().ok_or("invalid UTF16 unit")?)?))
        .collect()
}
fn size(value: &Value) -> Result<CellSize, Box<dyn Error>> {
    Ok(CellSize {
        width: number(&value["width"])?,
        height: number(&value["height"])?,
    })
}
fn geometry(value: &Value) -> Result<IipGeometry, Box<dyn Error>> {
    Ok(IipGeometry {
        columns: number(&value["columns"])?,
        rows: number(&value["rows"])?,
        render: if value["render"].is_null() {
            None
        } else {
            Some(RenderMetrics {
                cell: size(&value["render"]["cell"])?,
                canvas: size(&value["render"]["canvas"])?,
            })
        },
        device_pixel_ratio: if value["dpr"].is_null() {
            None
        } else {
            Some(number(&value["dpr"])?)
        },
    })
}
fn fields(fields: &HeaderFields) -> Value {
    // Same tagged wire values as the separate header oracle, not JS coercion.
    json!(fields
        .entries()
        .iter()
        .map(|(key, value)| {
            let value = match value {
                HeaderValue::Number(v) => json!(["n", bits(*v)]),
                HeaderValue::Text(v) => json!(["s", v]),
                HeaderValue::Codepoints(v) => json!(["a", v]),
                HeaderValue::Null => json!(["null"]),
                HeaderValue::Undefined => json!(["u"]),
                HeaderValue::Boolean(v) => json!(["b", v]),
                HeaderValue::DecoderObject => json!(["o"]),
            };
            json!([key, value])
        })
        .collect::<Vec<_>>())
}
fn effect(effect: IipEffect) -> Value {
    match effect {
        IipEffect::Report(units) => json!(["report", units, false]),
        IipEffect::Warning(text) => json!(["warning", text]),
        IipEffect::HostWarning { message, cause } => json!(["warning", message, cause.to_string()]),
        IipEffect::Display(request) => json!([
            "request",
            request.bytes,
            request.source.mime.mime(),
            request.source.width,
            request.source.height,
            bits(request.target.width),
            bits(request.target.height)
        ]),
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut handler = IipHandler::new(IipOptions::default(), NameDecodePolicy::NodeBuffer)?;
    for line in io::stdin().lock().lines() {
        let input: Value = serde_json::from_str(&line?)?;
        let mut effects = Vec::new();
        let mut error = None;
        match input["op"].as_str().ok_or("missing op")? {
            "new" => {
                handler = IipHandler::new(
                    IipOptions {
                        pixel_limit: number(&input["pixelLimit"])?,
                        size_limit: usize::try_from(
                            input["sizeLimit"].as_u64().ok_or("missing size limit")?,
                        )?,
                    },
                    NameDecodePolicy::NodeBuffer,
                )?
            }
            "start" => handler.start(),
            "reset" => handler.reset(),
            "put" => {
                let data = input["data"]
                    .as_array()
                    .ok_or("missing data")?
                    .iter()
                    .map(|v| Ok(u32::try_from(v.as_u64().ok_or("invalid codepoint")?)?))
                    .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
                handler.put(
                    &data,
                    usize::try_from(input["start"].as_u64().ok_or("missing start")?)?,
                    usize::try_from(input["end"].as_u64().ok_or("missing end")?)?,
                )?;
            }
            "end" => {
                let success = input["success"].as_bool().ok_or("missing success flag")?;
                match handler.prepare_end(success)? {
                    EndPreparation::Complete(completed) => effects = completed,
                    EndPreparation::Geometry(token) => {
                        if input["resetDuringGeometry"] == true {
                            handler.reset();
                        }
                        if input["throwDuringGeometry"] == true {
                            error = Some("geometry getter threw");
                        } else {
                            effects =
                                handler.finish_geometry(token, &geometry(&input["geometry"])?)?;
                        }
                    }
                }
            }
            "resize" => {
                let request = SizeRequest {
                    width: units(&input["width"])?,
                    height: units(&input["height"])?,
                    preserve_aspect_ratio: number(&input["ratio"])?,
                };
                let result = geometry(&input["geometry"])?.resize(
                    number(&input["sourceWidth"])?,
                    number(&input["sourceHeight"])?,
                    &request,
                );
                println!("{}", json!([bits(result.width), bits(result.height)]));
                continue;
            }
            "fixed" => {
                println!("{}", json!(fixed_three(number(&input["value"])?)));
                continue;
            }
            _ => return Err("unknown IIP replay operation".into()),
        }
        let parser = handler.header_parser();
        println!(
            "{}",
            json!({"aborted":handler.is_aborted(),"multipart":handler.is_multipart(),"abortMultipart":handler.is_multipart_aborted(),"parserState":parser.state as u8,"fields":fields(&parser.fields),"header":fields(handler.active_header()),"loadedBytes":handler.loaded_bytes(),"effects":effects.into_iter().map(effect).collect::<Vec<_>>(),"error":error})
        );
    }
    Ok(())
}
