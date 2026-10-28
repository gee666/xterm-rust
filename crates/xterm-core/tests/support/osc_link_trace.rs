// Copyright (c) 2022 The xterm.js authors. All rights reserved. MIT license.
// Shared by the fixture assertion and executable differential adapter.
use std::{cell::RefCell, collections::BTreeSet, error::Error, rc::Rc};
use xterm_core::{
    buffer::{marker::Marker, AttributeData, BufferService},
    options::{OptionValue, OptionsService},
    osc_link_service::{LinkDataRef, OscLinkData, OscLinkService},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub struct Runner {
    buffers: Rc<BufferService>,
    links: OscLinkService,
    data: Vec<LinkDataRef>,
    ids: BTreeSet<u64>,
    markers: Vec<Marker>,
}

impl Runner {
    pub fn new() -> Result<Self> {
        let options = OptionsService::new(
            false,
            [
                ("cols", OptionValue::Number(10.0)),
                ("rows", OptionValue::Number(3.0)),
                ("scrollback", OptionValue::Number(2.0)),
            ],
        );
        let buffers = Rc::new(BufferService::new(Rc::new(options))?);
        let links = OscLinkService::new(Rc::clone(&buffers));
        Ok(Self {
            buffers,
            links,
            data: Vec::new(),
            ids: BTreeSet::new(),
            markers: Vec::new(),
        })
    }

    pub fn step(&mut self, operation: &str) -> Result<String> {
        let fields: Vec<_> = operation.split(' ').collect();
        let field = |index| fields.get(index).copied().ok_or("missing trace field");
        let mut reply = "-".to_owned();
        match field(0)? {
            "data" => {
                self.data.push(Rc::new(RefCell::new(OscLinkData {
                    id: decode_id(field(1)?)?,
                    uri: decode(field(2)?)?,
                })));
            }
            "mut" => {
                let index: usize = field(1)?.parse()?;
                let mut data = self.data.get(index).ok_or("unknown data")?.borrow_mut();
                data.id = decode_id(field(2)?)?;
                data.uri = decode(field(3)?)?;
            }
            "reg" => {
                let index: usize = field(1)?.parse()?;
                let id = self
                    .links
                    .register_link(Rc::clone(self.data.get(index).ok_or("unknown data")?))?;
                self.ids.insert(id);
                reply = id.to_string();
            }
            "line" => self
                .links
                .add_line_to_link(field(1)?.parse()?, field(2)?.parse()?)?,
            "cursor" => {
                let active = self.buffers.buffer();
                let mut active = active.borrow_mut();
                active.ybase = field(1)?.parse()?;
                active.y = field(2)?.parse()?;
            }
            "alt" => self.buffers.buffers.activate_alt_buffer(None)?,
            "normal" => self.buffers.buffers.activate_normal_buffer()?,
            "scroll" => self.buffers.scroll(&AttributeData::default(), false)?,
            "dispose" => {
                let index: usize = field(1)?.parse()?;
                if let Some(marker) = self.markers.get(index) {
                    marker.dispose()?;
                }
            }
            "move" => {
                let index: usize = field(1)?.parse()?;
                if let Some(marker) = self.markers.get(index) {
                    marker.set_line(field(2)?.parse()?);
                }
            }
            _ => return Err("unknown trace operation".into()),
        }
        for buffer in [self.buffers.buffers.normal(), self.buffers.buffers.alt()] {
            let buffer = buffer.borrow();
            for marker in buffer.markers.borrow().iter() {
                if !self.markers.iter().any(|old| old.ptr_eq(marker)) {
                    self.markers.push(marker.clone());
                }
            }
        }
        let normal = Rc::ptr_eq(&self.buffers.buffer(), &self.buffers.buffers.normal());
        let mut result = format!("{reply};{};", if normal { "n" } else { "a" });
        for id in &self.ids {
            result.push_str(&format!("{id}:"));
            if let Some(data) = self.links.get_link_data(*id) {
                let data = data.borrow();
                result.push_str(&data.id.as_deref().map(encode).unwrap_or_else(|| "-".into()));
                result.push('/');
                result.push_str(&encode(&data.uri));
            } else {
                result.push('-');
            }
            result.push(',');
        }
        result.push(';');
        for marker in &self.markers {
            result.push_str(&format!(
                "{}:{},",
                marker.line(),
                u8::from(marker.is_disposed())
            ));
        }
        Ok(result)
    }
}

fn decode_id(text: &str) -> Result<Option<Vec<u16>>> {
    if text == "-" {
        Ok(None)
    } else {
        Ok(Some(decode(text)?))
    }
}
fn decode(text: &str) -> Result<Vec<u16>> {
    if text == "_" {
        return Ok(Vec::new());
    }
    if !text.is_ascii() || text.len() % 4 != 0 {
        return Err("invalid UTF16 trace field".into());
    }
    (0..text.len())
        .step_by(4)
        .map(|i| u16::from_str_radix(&text[i..i + 4], 16).map_err(Into::into))
        .collect()
}
fn encode(units: &[u16]) -> String {
    if units.is_empty() {
        return "_".into();
    }
    units.iter().map(|unit| format!("{unit:04x}")).collect()
}
