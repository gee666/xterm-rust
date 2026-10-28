// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Shared registration disposal while an outer terminal/parser call is active.
use super::super::{
    apc::ApcDisposalHandle, dcs::DcsDisposalHandle, handler::registry::Registrar,
    osc::OscDisposalHandle,
};
use super::{ExecuteHandler, HandlerResult, Params, Registry};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
};

pub(super) type ExecuteHandle = Rc<RefCell<ExecuteHandler>>;
pub(super) type ExecuteStorage = RefCell<HashMap<u32, ExecuteHandle>>;
pub(super) type SharedExecute = Rc<ExecuteStorage>;
type Commands = Registrar<u32, dyn FnMut(&Params) -> HandlerResult>;
struct Control {
    disposed: Cell<bool>,
    execute: Weak<ExecuteStorage>,
    csi: Commands,
    esc: Commands,
    osc: OscDisposalHandle,
    dcs: DcsDisposalHandle,
    apc: ApcDisposalHandle,
}

/// The parent clears its registered resources only once, like DisposableStore.
/// Weak child/registry handles never extend callback or parser lifetime.
#[derive(Clone)]
pub struct ParserDisposalHandle(Rc<Control>);
impl ParserDisposalHandle {
    pub(super) fn new(
        execute: &SharedExecute,
        csi: &Registry,
        esc: &Registry,
        osc: OscDisposalHandle,
        dcs: DcsDisposalHandle,
        apc: ApcDisposalHandle,
    ) -> Self {
        Self(Rc::new(Control {
            disposed: Cell::new(false),
            execute: Rc::downgrade(execute),
            csi: csi.registrar(),
            esc: esc.registrar(),
            osc,
            dcs,
            apc,
        }))
    }
    pub fn dispose(&self) {
        if self.0.disposed.replace(true) {
            return;
        }
        // These Options describe already-dropped weak owners, not callback
        // errors. Source disposal performs no end/reset callbacks.
        let _csi_owner_exists = self.0.csi.clear_all();
        if let Some(execute) = self.0.execute.upgrade() {
            let removed = std::mem::take(&mut *execute.borrow_mut());
            drop(removed);
        }
        let _esc_owner_exists = self.0.esc.clear_all();
        let _osc_owner_exists = self.0.osc.dispose();
        let _dcs_owner_exists = self.0.dcs.dispose();
        let _apc_owner_exists = self.0.apc.dispose();
    }
    pub fn is_disposed(&self) -> bool {
        self.0.disposed.get()
    }
}
