//! Line protocol for deterministic differential checks. Not an interactive demo.
use std::cell::RefCell;
use std::error::Error;
use std::io::{self, BufRead};
use std::rc::Rc;
use xterm_core::circular_list::CircularList;
use xterm_core::lifecycle::Disposable;

type List = Rc<CircularList<i32>>;
type Events = Rc<RefCell<Vec<String>>>;

fn values(list: &CircularList<i32>) -> String {
    (0..list.max_length() * 2 + 2)
        .map(|index| {
            list.get(index)
                .map_or_else(|| "_".into(), |value| value.to_string())
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn make_list(capacity: usize, events: &Events) -> Result<List, Box<dyn Error>> {
    let list = Rc::new(CircularList::new(capacity)?);
    let target = Rc::downgrade(&list);
    let output = Rc::clone(events);
    list.on_delete_emitter.event().subscribe(move |change| {
        if let Some(list) = target.upgrade() {
            output.borrow_mut().push(format!(
                "D,{},{},{},{}",
                change.index,
                change.amount,
                list.len(),
                values(&list)
            ));
        }
    });
    let target = Rc::downgrade(&list);
    let output = Rc::clone(events);
    list.on_insert_emitter.event().subscribe(move |change| {
        if let Some(list) = target.upgrade() {
            output.borrow_mut().push(format!(
                "I,{},{},{},{}",
                change.index,
                change.amount,
                list.len(),
                values(&list)
            ));
        }
    });
    let target = Rc::downgrade(&list);
    let output = Rc::clone(events);
    list.on_trim_emitter.event().subscribe(move |amount| {
        if let Some(list) = target.upgrade() {
            output
                .borrow_mut()
                .push(format!("T,{amount},{},{}", list.len(), values(&list)));
        }
    });
    Ok(list)
}

fn main() -> Result<(), Box<dyn Error>> {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut list = make_list(5, &events)?;
    for line in io::stdin().lock().lines() {
        let line = line?;
        let fields: Vec<_> = line.split('\t').collect();
        let integer = |index: usize| -> Result<usize, Box<dyn Error>> {
            Ok(fields.get(index).ok_or("missing protocol field")?.parse()?)
        };
        let optional =
            |value: Option<i32>| value.map_or_else(|| "_".into(), |value| value.to_string());
        let mut result = "ok".to_string();
        match fields[0] {
            "N" => list = make_list(integer(1)?, &events)?,
            "P" => list.push(fields[1].parse()?)?,
            "S" => list.set(
                integer(1)?,
                if fields[2] == "_" {
                    None
                } else {
                    Some(fields[2].parse()?)
                },
            )?,
            "Q" => list.set_signed(
                fields[1].parse()?,
                if fields[2] == "_" {
                    None
                } else {
                    Some(fields[2].parse()?)
                },
            )?,
            "G" => result = optional(list.get_signed(fields[1].parse()?)),
            "L" => list.set_length(integer(1)?),
            "M" => list.set_max_length(integer(1)?)?,
            "O" => result = optional(list.pop()?),
            "R" => {
                result = match list.recycle() {
                    Ok(value) => optional(value),
                    Err(error) => error.to_string(),
                }
            }
            "D" => list.splice(
                integer(1)?,
                integer(2)?,
                &fields[3..]
                    .iter()
                    .map(|value| value.parse())
                    .collect::<Result<Vec<i32>, _>>()?,
            )?,
            "T" => list.trim_start(integer(1)?)?,
            "H" => {
                if let Err(error) =
                    list.shift_elements(fields[1].parse()?, integer(2)?, fields[3].parse()?)
                {
                    result = error.to_string();
                }
            }
            "X" => list.dispose(),
            command => return Err(format!("unknown protocol command {command}").into()),
        }
        println!(
            "{}|{}|{}|{}|{}|{}",
            list.len(),
            list.max_length(),
            list.is_full(),
            result,
            values(&list),
            events.borrow().join(";")
        );
        events.borrow_mut().clear();
    }
    Ok(())
}
