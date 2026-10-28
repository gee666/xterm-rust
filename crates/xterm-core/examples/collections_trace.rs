//! Line protocol used by scripts/collections-oracle.mjs, not a terminal demo.
use std::error::Error;
use std::io::{self, BufRead};
use xterm_core::multi_key_map::{FourKeyMap, PropertyKey, TwoKeyMap};
use xterm_core::sorted_list::SortedList;

fn missing() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, "missing trace argument")
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut list = SortedList::new(|value: &i32| f64::from(*value));
    let mut map = TwoKeyMap::new();
    for line in io::stdin().lock().lines() {
        let line = line?;
        let mut parts = line.split('\t');
        match parts.next().ok_or_else(missing)? {
            "I" => {
                if list
                    .insert(parts.next().ok_or_else(missing)?.parse()?)
                    .is_err()
                {
                    println!("\"RangeError\"");
                }
            }
            "D" => println!(
                "{}",
                list.delete(&parts.next().ok_or_else(missing)?.parse()?)?
            ),
            "C" => list.clear(),
            "V" => match list.values() {
                Ok(values) => println!("{:?}", values.collect::<Vec<_>>()),
                Err(_) => println!("\"RangeError\""),
            },
            "K" => match list.get_key_iterator(parts.next().ok_or_else(missing)?.parse()?) {
                Ok(values) => println!("{:?}", values.collect::<Vec<_>>()),
                Err(_) => println!("\"RangeError\""),
            },
            "N" => {
                let key: f64 = parts.next().ok_or_else(missing)?.parse()?;
                map.set(key, "key", 1);
            }
            "B" => {
                let bits = u64::from_str_radix(parts.next().ok_or_else(missing)?, 16)?;
                map.set(f64::from_bits(bits), "key", 1);
            }
            "Q" => {
                let bits = u64::from_str_radix(parts.next().ok_or_else(missing)?, 16)?;
                let position: usize = parts.next().ok_or_else(missing)?.parse()?;
                if position >= 4 {
                    return Err(missing().into());
                }
                let text = parts.next().ok_or_else(missing)?;
                let mut keys = [1, 2, 3, 4].map(PropertyKey::from);
                keys[position] = f64::from_bits(bits).into();
                let mut four = FourKeyMap::new();
                four.set(
                    keys[0].clone(),
                    keys[1].clone(),
                    keys[2].clone(),
                    keys[3].clone(),
                    1,
                );
                let mut query = ["1", "2", "3", "4"].map(str::to_owned);
                query[position] = text.to_owned();
                println!(
                    "{}",
                    four.get(
                        query[0].clone(),
                        query[1].clone(),
                        query[2].clone(),
                        query[3].clone()
                    )
                    .is_some()
                );
            }
            "S" => {
                let key = parts.next().ok_or_else(missing)?;
                println!("{}", map.get(key, "key").is_some());
                map.clear();
            }
            other => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown trace operation {other}"),
                )
                .into())
            }
        }
    }
    Ok(())
}
