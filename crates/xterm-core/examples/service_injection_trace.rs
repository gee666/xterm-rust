//! Oracle protocol for original ServiceRegistry and InstantiationService traces.
use std::collections::HashMap;
use std::error::Error;
use std::io::{self, BufRead};
use std::rc::Rc;
use xterm_core::instantiation::{
    EmptyValue, InstantiationService, ServiceCollection, ServiceValue,
};
use xterm_core::service_registry::{ConstructorMetadata, ServiceId, ServiceRegistry};

fn field<'a>(fields: &mut impl Iterator<Item = &'a str>) -> Result<&'a str, Box<dyn Error>> {
    fields.next().ok_or_else(|| "missing oracle field".into())
}
fn truth<T: 'static>(value: T) -> Result<String, Box<dyn Error>> {
    let mut registry = ServiceRegistry::default();
    let service = InstantiationService::new(&mut registry)?;
    let id = registry.create_decorator::<T>("value")?;
    service.set_service(&id, Rc::new(value));
    let constructor = ConstructorMetadata::new("Ctor", None);
    constructor.decorate(id.key(), 0, 3)?;
    Ok(service
        .create_instance(&constructor, vec![], |_| "ok".to_owned())
        .unwrap_or_else(|e| e.to_string()))
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut registry = ServiceRegistry::default();
    let service = InstantiationService::new(&mut registry)?;
    let collection = ServiceCollection::default();
    let mut ids: HashMap<String, ServiceId<String>> = HashMap::new();
    let mut constructors: HashMap<String, ConstructorMetadata> = HashMap::new();
    for line in io::stdin().lock().lines() {
        let line = line?;
        let mut fields = line.split('\t');
        let command = field(&mut fields)?;
        let output = match command {
            "V" => match field(&mut fields)? {
                "undefined" => truth(EmptyValue::Undefined)?,
                "null" => truth(EmptyValue::Null)?,
                "false" => truth(false)?,
                "true" => truth(true)?,
                "zero" => truth(0.0_f64)?,
                "negative-zero" => truth(-0.0_f64)?,
                "nan" => truth(f64::NAN)?,
                "infinity" => truth(f64::INFINITY)?,
                "empty" => truth(String::new())?,
                "text" => truth("text".to_owned())?,
                "object" => truth(Vec::<u8>::new())?,
                _ => return Err("unknown oracle value".into()),
            },
            "R" => {
                let name = field(&mut fields)?;
                let id = registry.create_decorator::<String>(name)?;
                let same = ids.get(name).is_some_and(|old| old.key() == id.key());
                ids.insert(name.to_owned(), id);
                same.to_string()
            }
            "S" => {
                let id = ids.get(field(&mut fields)?).ok_or("unknown id")?;
                let value = Rc::new(field(&mut fields)?.to_owned());
                let old = collection.set(id, value.clone());
                service.set_service(id, value);
                old.map_or_else(|| "undefined".to_owned(), |s| (*s).clone())
            }
            "G" => {
                let id = ids.get(field(&mut fields)?).ok_or("unknown id")?;
                collection
                    .get(id)
                    .map_or_else(|| "undefined".to_owned(), |s| (*s).clone())
            }
            "H" => {
                let id = ids.get(field(&mut fields)?).ok_or("unknown id")?;
                collection.has(&id.key()).to_string()
            }
            "E" => {
                let mut result = Vec::new();
                collection.for_each(|id, value| {
                    if let Some(value) = value.downcast::<String>() {
                        result.push(format!("{}={value}", id.name()));
                    }
                });
                result.join(",")
            }
            "L" => {
                let first = ids.get("a").ok_or("missing a")?;
                let second = ids.get("b").ok_or("missing b")?;
                let added = ids.get("d").ok_or("missing d")?;
                let mut result = Vec::new();
                collection.for_each(|key, value| {
                    if let Some(value) = value.downcast::<String>() {
                        result.push(format!("{}={value}", key.name()));
                    }
                    if key == first.key() {
                        collection.set(second, Rc::new("Bnext".to_owned()));
                        collection.set(added, Rc::new("D".to_owned()));
                        let mut count = 0;
                        collection.for_each(|_, _| count += 1);
                        result.push(format!("nested={count}"));
                    }
                });
                result.join(",")
            }
            "N" => {
                let name = field(&mut fields)?;
                let parent_name = field(&mut fields)?;
                let parent = if parent_name == "-" {
                    None
                } else {
                    Some(
                        constructors
                            .get(parent_name)
                            .ok_or("unknown parent")?
                            .clone(),
                    )
                };
                constructors.insert(name.to_owned(), ConstructorMetadata::new(name, parent));
                "ok".to_owned()
            }
            "D" => {
                let constructor = constructors
                    .get(field(&mut fields)?)
                    .ok_or("unknown constructor")?;
                let id = ids.get(field(&mut fields)?).ok_or("unknown id")?;
                let index = field(&mut fields)?.parse()?;
                let arity = field(&mut fields)?.parse()?;
                constructor
                    .decorate(id.key(), index, arity)
                    .map_or_else(|e| e.to_string(), |_| "ok".to_owned())
            }
            "M" => {
                let constructor = constructors
                    .get(field(&mut fields)?)
                    .ok_or("unknown constructor")?;
                constructor
                    .dependencies()
                    .iter()
                    .map(|d| format!("{}@{}", d.id.name(), d.index))
                    .collect::<Vec<_>>()
                    .join(",")
            }
            "I" => {
                let constructor = constructors
                    .get(field(&mut fields)?)
                    .ok_or("unknown constructor")?;
                let count: usize = field(&mut fields)?.parse()?;
                let args = (0..count)
                    .map(|i| ServiceValue::new(format!("static{i}")))
                    .collect();
                service
                    .create_instance(constructor, args, |args| {
                        args.iter()
                            .filter_map(|a| a.downcast::<String>())
                            .map(|s| (*s).clone())
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .unwrap_or_else(|error| error.to_string())
            }
            _ => return Err("unknown oracle command".into()),
        };
        println!("{output}");
    }
    Ok(())
}
