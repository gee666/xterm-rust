// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! OSC title, color and hyperlink callbacks over the actual terminal state.
use super::{TerminalError, TerminalEvent, TerminalState};
use crate::{
    osc_link_service::OscLinkData,
    parser::{
        escape::EscapeSequenceParser,
        handler::{DispatchError, HandlerResult},
        osc::OscHandler,
    },
};
use std::{cell::RefCell, rc::Rc};

pub(super) fn register(parser: &mut EscapeSequenceParser, state: &Rc<RefCell<TerminalState>>) {
    for ident in [0, 1, 2, 4, 8, 10, 11, 12, 104, 110, 111, 112] {
        let shared = Rc::clone(state);
        parser.osc.register_handler(
            f64::from(ident),
            Rc::new(RefCell::new(OscHandler::new(move |data| {
                match handle(&shared, ident, data) {
                    Ok(handled) => HandlerResult::Ready(handled),
                    Err(error) => {
                        if let Ok(mut state) = shared.try_borrow_mut() {
                            state.error = Some(error);
                        }
                        HandlerResult::Failed(DispatchError::CallbackFailed)
                    }
                }
            }))),
        );
    }
}

fn handle(
    shared: &Rc<RefCell<TerminalState>>,
    ident: u16,
    data: Vec<u16>,
) -> Result<bool, TerminalError> {
    if ident == 0 || ident == 2 {
        shared
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?
            .set_title(data.clone())?;
        // OSC0 observers see the previous icon. Do not batch its mutation before
        // dispatch, and do not hold state across potentially reentrant callbacks.
        super::dispatch_events(shared)?;
    }
    if ident == 0 || ident == 1 {
        shared
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?
            .set_icon_name(data)?;
        return Ok(true);
    }
    if ident == 2 {
        return Ok(true);
    }
    if ident == 8 {
        return shared
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?
            .set_hyperlink(&data);
    }
    for batch in super::osc_colors::requests(ident, &data) {
        shared
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?
            .events
            .borrow_mut()
            .push(TerminalEvent::Color(batch));
        super::dispatch_events(shared)?;
    }
    Ok(true)
}

impl TerminalState {
    pub(crate) fn set_title(&mut self, data: Vec<u16>) -> Result<(), TerminalError> {
        self.title = data.clone();
        self.events
            .borrow_mut()
            .push(TerminalEvent::TitleChange(data));
        Ok(())
    }

    pub(crate) fn set_icon_name(&mut self, data: Vec<u16>) -> Result<(), TerminalError> {
        self.icon_name = data;
        Ok(())
    }

    pub(crate) fn set_hyperlink(&mut self, data: &[u16]) -> Result<bool, TerminalError> {
        let Some(separator) = data.iter().position(|&unit| unit == u16::from(b';')) else {
            return Ok(true);
        };
        let params = trim(&data[..separator]);
        let uri = &data[separator + 1..];
        if uri.is_empty() {
            if !params.is_empty() {
                return Ok(false);
            }
            self.finish_hyperlink();
            return Ok(true);
        }
        if self.attributes.extended.url_id() != 0 {
            self.finish_hyperlink();
        }
        let id = params
            .split(|&unit| unit == u16::from(b':'))
            .find(|part| part.starts_with(&[u16::from(b'i'), u16::from(b'd'), u16::from(b'=')]))
            .map(|part| &part[3..])
            .filter(|id| !id.is_empty())
            .map(<[u16]>::to_vec);
        // Clone before registering, matching source partial state on an error.
        self.attributes.extended = self.attributes.extended.clone();
        let id = self.links.register_link(Rc::new(RefCell::new(OscLinkData {
            id,
            uri: uri.to_vec(),
        })))?;
        self.attributes.extended.set_url_id(id);
        self.attributes.update_extended();
        Ok(true)
    }

    fn finish_hyperlink(&mut self) {
        self.attributes.extended = self.attributes.extended.clone();
        self.attributes.extended.set_url_id(0);
        self.attributes.update_extended();
    }
}

fn trim(mut units: &[u16]) -> &[u16] {
    while units.first().is_some_and(|&unit| whitespace(unit)) {
        units = &units[1..];
    }
    while units.last().is_some_and(|&unit| whitespace(unit)) {
        units = &units[..units.len() - 1];
    }
    units
}
fn whitespace(unit: u16) -> bool {
    // ECMAScript trim, including BOM but excluding NEL and lone surrogates.
    matches!(unit, 0x9..=0xd | 0x20 | 0xa0 | 0x1680 | 0x2000..=0x200a | 0x2028 | 0x2029 | 0x202f | 0x205f | 0x3000 | 0xfeff)
}
