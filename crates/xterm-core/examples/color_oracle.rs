//! Line protocol used by tools/check_color.mjs, not a terminal demonstration.
use std::io::{self, BufRead};
use xterm_core::color::{channels, css, rgb, rgba, value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for line in io::stdin().lock().lines() {
        let line = line?;
        let parts: Vec<_> = line.split('\t').collect();
        let number = |index: usize| -> Result<u32, Box<dyn std::error::Error>> {
            Ok(parts.get(index).ok_or("missing color field")?.parse()?)
        };
        let ratio = |index: usize| -> Result<f64, Box<dyn std::error::Error>> {
            Ok(parts.get(index).ok_or("missing numeric field")?.parse()?)
        };
        let bg = number(1);
        match parts[0] {
            "blend" => println!("{}", rgba::blend(bg?, number(2)?)),
            "ensure" => match rgba::ensure_contrast_ratio(bg?, number(2)?, ratio(3)?) {
                Some(result) => println!("{result}"),
                None => println!("undefined"),
            },
            "reduce" => println!("{}", rgba::reduce_luminance(bg?, number(2)?, ratio(3)?)),
            "increase" => println!("{}", rgba::increase_luminance(bg?, number(2)?, ratio(3)?)),
            "luminance" => println!("{:.4}", rgb::relative_luminance(bg?)),
            "opacity" | "multiply" => {
                let [r, g, b, a] = bg?.to_be_bytes();
                let color = channels::to_color(r.into(), g.into(), b.into(), Some(a.into()));
                let result = if parts[0] == "opacity" {
                    value::opacity(&color, ratio(2)?)
                } else {
                    value::multiply_opacity(&color, ratio(2)?)
                };
                println!("{}|{}", result.css, result.rgba);
            }
            "parse" => match css::to_color(parts.get(1).ok_or("missing CSS field")?) {
                Ok(result) => println!("{}|{}", result.css, result.rgba),
                Err(_) => println!("error"),
            },
            _ => return Err("unknown color operation".into()),
        }
    }
    Ok(())
}
