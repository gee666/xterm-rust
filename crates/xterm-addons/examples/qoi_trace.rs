//! QOI decoder replay against the supplied WASM codec, not a browser renderer.
use serde_json::{json, Value};
use std::error::Error;
use std::io::{self, BufRead};
use xterm_addons::image::qoi::QoiDecoder;
fn runs(bytes: &[u8]) -> Vec<(u32, usize)> {
    let mut result = Vec::new();
    for pixel in bytes.chunks_exact(4) {
        let word = u32::from_le_bytes([pixel[0], pixel[1], pixel[2], pixel[3]]);
        match result.last_mut() {
            Some((last, count)) if *last == word => *count += 1,
            _ => result.push((word, 1)),
        }
    }
    result
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut decoder = QoiDecoder::new(4_194_304.0);
    for line in io::stdin().lock().lines() {
        let operation: Value = serde_json::from_str(&line?)?;
        let mut pixels = vec![];
        let mut error = None;
        match operation["op"].as_str().ok_or("missing operation")? {
            "new" => {
                decoder = QoiDecoder::new(operation["keep"].as_f64().ok_or("missing keep size")?)
            }
            "release" => decoder.release(),
            "decode" => {
                let data = if let Some(file) = operation["file"].as_str() {
                    let directory = match file {
                        "dice.qoi" => "tests/fixtures/image-metrics",
                        "palette.qoi" => "tests/fixtures/qoi",
                        _ => return Err("unknown fixture".into()),
                    };
                    std::fs::read(
                        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                            .join(directory)
                            .join(file),
                    )?
                } else {
                    operation["data"]
                        .as_array()
                        .ok_or("missing bytes")?
                        .iter()
                        .map(|v| Ok(u8::try_from(v.as_u64().ok_or("invalid byte")?)?))
                        .collect::<Result<Vec<_>, Box<dyn Error>>>()?
                };
                match decoder.decode(&data) {
                    Ok(image) => pixels = runs(&image.pixels),
                    Err(e) => error = Some(e.js_name()),
                }
            }
            _ => return Err("unknown operation".into()),
        }
        println!(
            "{}",
            json!({"width":decoder.width,"height":decoder.height,"memory":decoder.memory_usage(),"initialized":decoder.is_initialized(),"pixels":pixels,"error":error})
        );
    }
    Ok(())
}
