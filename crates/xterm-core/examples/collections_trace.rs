//! Line protocol used by scripts/collections-oracle.mjs, not a terminal demo.
use std::error::Error;
use std::io::{self, BufRead};
use xterm_core::multi_key_map::TwoKeyMap;
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
            "I" => list.insert(parts.next().ok_or_else(missing)?.parse()?),
            "D" => println!(
                "{}",
                list.delete(&parts.next().ok_or_else(missing)?.parse()?)
            ),
            "C" => list.clear(),
            "V" => println!("{:?}", list.values().collect::<Vec<_>>()),
            "K" => println!(
                "{:?}",
                list.get_key_iterator(parts.next().ok_or_else(missing)?.parse()?)
                    .collect::<Vec<_>>()
            ),
            "N" => {
                let key: f64 = parts.next().ok_or_else(missing)?.parse()?;
                map.set(key, "key", 1);
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
