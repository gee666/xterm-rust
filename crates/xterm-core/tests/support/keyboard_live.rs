// Licensed under MIT. Shared live getter fixture source for all keyboard encoders.
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::input::keyboard_event_source::KeyboardEventSource;

struct Sequence<T> {
    values: Vec<T>,
    offset: Cell<usize>,
}
impl<T: Clone> Sequence<T> {
    fn new(values: Vec<T>) -> Self {
        assert!(!values.is_empty());
        Self {
            values,
            offset: Cell::new(0),
        }
    }
    fn next(&self) -> T {
        let index = self.offset.replace(self.offset.get() + 1);
        self.values[index.min(self.values.len() - 1)].clone()
    }
}
#[derive(Debug)]
pub struct Marker;
pub struct Source {
    number: Sequence<f64>,
    key: Sequence<Vec<u16>>,
    code: Sequence<Vec<u16>>,
    shift: Sequence<bool>,
    alt: Sequence<bool>,
    ctrl: Sequence<bool>,
    meta: Sequence<bool>,
    throw_at: usize,
    pub reads: RefCell<String>,
    pub marker: Rc<Marker>,
}
impl Source {
    fn read(&self, letter: char) -> Result<(), Rc<Marker>> {
        let length = {
            let mut reads = self.reads.borrow_mut();
            reads.push(letter);
            reads.len()
        };
        if length == self.throw_at {
            Err(self.marker.clone())
        } else {
            Ok(())
        }
    }
    pub fn from_fields(fields: &[&str], first_value: usize, throw_index: usize) -> Self {
        let bools = |text: &str| {
            Sequence::new(
                text.chars()
                    .map(|value| match value {
                        '0' => false,
                        '1' => true,
                        _ => panic!("invalid fixture boolean"),
                    })
                    .collect(),
            )
        };
        let strings = |text: &str| Sequence::new(text.split(';').map(units).collect());
        Self {
            number: Sequence::new(fields[first_value].split(';').map(number).collect()),
            key: strings(fields[first_value + 1]),
            code: strings(fields[first_value + 2]),
            shift: bools(fields[first_value + 3]),
            alt: bools(fields[first_value + 4]),
            ctrl: bools(fields[first_value + 5]),
            meta: bools(fields[first_value + 6]),
            throw_at: fields[throw_index].parse().unwrap(),
            reads: RefCell::new(String::new()),
            marker: Rc::new(Marker),
        }
    }
}
impl KeyboardEventSource for Source {
    type Error = Rc<Marker>;
    fn key_code(&self) -> Result<f64, Self::Error> {
        self.read('n')?;
        Ok(self.number.next())
    }
    fn key(&self) -> Result<Cow<'_, [u16]>, Self::Error> {
        self.read('k')?;
        Ok(Cow::Owned(self.key.next()))
    }
    fn code(&self) -> Result<Cow<'_, [u16]>, Self::Error> {
        self.read('d')?;
        Ok(Cow::Owned(self.code.next()))
    }
    fn shift_key(&self) -> Result<bool, Self::Error> {
        self.read('s')?;
        Ok(self.shift.next())
    }
    fn alt_key(&self) -> Result<bool, Self::Error> {
        self.read('a')?;
        Ok(self.alt.next())
    }
    fn ctrl_key(&self) -> Result<bool, Self::Error> {
        self.read('c')?;
        Ok(self.ctrl.next())
    }
    fn meta_key(&self) -> Result<bool, Self::Error> {
        self.read('m')?;
        Ok(self.meta.next())
    }
}
fn number(value: &str) -> f64 {
    match value {
        "NaN" => f64::NAN,
        "Infinity" => f64::INFINITY,
        "-Infinity" => f64::NEG_INFINITY,
        _ => value.parse().unwrap(),
    }
}
pub fn units(value: &str) -> Vec<u16> {
    if value == "-" {
        return Vec::new();
    }
    value
        .split(',')
        .map(|unit| u16::from_str_radix(unit, 16).unwrap())
        .collect()
}
