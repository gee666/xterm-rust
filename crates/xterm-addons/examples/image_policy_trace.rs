//! Replay image option/control policy; no DOM or graphics rendering is claimed.
use serde_json::{json, Value};
use std::error::Error;
use std::io::{self, BufRead};
use xterm_addons::image::options::ImageOptions;
use xterm_addons::image::policy::{
    GraphicsGeometry, ImageControlAction, ImageControlEffect, ImageControlPolicy, ImageParameter,
};
use xterm_addons::image::storage::CellSize;
fn number(value: &Value) -> Result<f64, Box<dyn Error>> {
    Ok(f64::from_bits(u64::from_str_radix(
        value.as_str().ok_or("missing float bits")?,
        16,
    )?))
}
fn bits(value: f64) -> String {
    if value.is_nan() {
        "NaN".into()
    } else {
        format!("{:016x}", value.to_bits())
    }
}
fn configure(options: &mut ImageOptions, values: &Value) -> Result<(), Box<dyn Error>> {
    let Some(values) = values.as_object() else {
        return Ok(());
    };
    for (key, value) in values {
        match key.as_str() {
            "enableSizeReports" => {
                options.enable_size_reports = value.as_bool().ok_or("invalid flag")?
            }
            "showPlaceholder" => {
                options.show_placeholder = value.as_bool().ok_or("invalid flag")?
            }
            "sixelSupport" => options.sixel_support = value.as_bool().ok_or("invalid flag")?,
            "sixelScrolling" => options.sixel_scrolling = value.as_bool().ok_or("invalid flag")?,
            "iipSupport" => options.iip_support = value.as_bool().ok_or("invalid flag")?,
            "kittySupport" => options.kitty_support = value.as_bool().ok_or("invalid flag")?,
            "pixelLimit" => options.pixel_limit = number(value)?,
            "storageLimit" => options.storage_limit = number(value)?,
            "sixelPaletteLimit" => options.sixel_palette_limit = number(value)?,
            "sixelSizeLimit" => options.sixel_size_limit = number(value)?,
            "iipSizeLimit" => options.iip_size_limit = number(value)?,
            "kittySizeLimit" => options.kitty_size_limit = number(value)?,
            _ => return Err("unknown image option".into()),
        }
    }
    Ok(())
}
fn options(value: ImageOptions) -> Value {
    json!([
        value.enable_size_reports,
        bits(value.pixel_limit),
        bits(value.storage_limit),
        value.show_placeholder,
        value.sixel_support,
        value.sixel_scrolling,
        bits(value.sixel_palette_limit),
        bits(value.sixel_size_limit),
        value.iip_support,
        bits(value.iip_size_limit),
        value.kitty_support,
        bits(value.kitty_size_limit)
    ])
}
fn parameters(input: &Value) -> Result<Vec<ImageParameter>, Box<dyn Error>> {
    input
        .as_array()
        .ok_or("missing parameters")?
        .iter()
        .map(|value| {
            Ok(match value[0].as_str().ok_or("missing parameter kind")? {
                "n" => ImageParameter::Number(number(&value[1])?),
                "a" => ImageParameter::Subparams(
                    value[1]
                        .as_array()
                        .ok_or("missing subparams")?
                        .iter()
                        .map(number)
                        .collect::<Result<Vec<_>, _>>()?,
                ),
                "u" => ImageParameter::Undefined,
                _ => return Err("bad parameter kind".into()),
            })
        })
        .collect()
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut policy = ImageControlPolicy::new(ImageOptions::default());
    let mut handlers = true;
    for line in io::stdin().lock().lines() {
        let input: Value = serde_json::from_str(&line?)?;
        let mut action = ImageControlAction {
            handled: false,
            effects: vec![],
        };
        match input["op"].as_str().ok_or("missing operation")? {
            "new" => {
                let mut value = ImageOptions::default();
                configure(&mut value, &input["options"])?;
                policy = ImageControlPolicy::new(value);
                handlers = true;
            }
            "set" => configure(&mut policy.options, &input["options"])?,
            "reset" => action = policy.reset(),
            "decset" => action = policy.decset(&parameters(&input["params"])?),
            "decrst" => action = policy.decrst(&parameters(&input["params"])?),
            "da" => action = policy.primary_device_attributes(&parameters(&input["params"])?),
            "graphics" => {
                let geometry = &input["geometry"];
                let canvas = if geometry["canvas"].is_null() {
                    None
                } else {
                    Some(CellSize {
                        width: number(&geometry["canvas"][0])?,
                        height: number(&geometry["canvas"][1])?,
                    })
                };
                action = policy.graphics_attributes(
                    &parameters(&input["params"])?,
                    GraphicsGeometry {
                        canvas,
                        columns: number(&geometry["columns"])?,
                        rows: number(&geometry["rows"])?,
                    },
                );
            }
            _ => return Err("unknown policy operation".into()),
        }
        let mut effects = Vec::new();
        for effect in action.effects {
            match effect {
                ImageControlEffect::Reply(reply) => {
                    effects.push(json!(["reply", reply, false]));
                    configure(&mut policy.options, &input["duringReply"])?;
                    if input["clearHandlersDuringReply"] == true {
                        handlers = false;
                    }
                }
                ImageControlEffect::ClearStorage => effects.push(json!([
                    "clear",
                    bits(policy.options.sixel_palette_limit),
                    policy.options.sixel_scrolling
                ])),
                ImageControlEffect::ResetHandlers => {
                    if handlers {
                        effects.push(json!([
                            "reset",
                            bits(policy.options.sixel_palette_limit),
                            policy.options.sixel_scrolling
                        ]));
                    }
                }
            }
        }
        println!(
            "{}",
            json!({"handled":action.handled,"effects":effects,"options":options(policy.options),"defaults":options(policy.constructor_defaults())})
        );
    }
    Ok(())
}
