// Copyright (c) 2022 The xterm.js authors. All rights reserved. MIT license.
// Host-language operators and ownership edges only. Registration, indexing,
// rendering and disposal execute in the Rust decoration service.
const owners = new WeakMap();
const records = new WeakMap();
function prepare(options) {
  const values = [options.x ?? 0, options.width ?? 0, options.height ?? 0];
  for (const value of values) {
    if (value && (value === Infinity || isNaN(value) || value % 1 !== 0 || value < 0)) {
      throw new Error('This API only accepts positive integers');
    }
  }
  if (options.marker.isDisposed) return;
  const record = { options, marker: options.marker, element: undefined };
  // Preserve the constructor's separate marker and overview option reads.
  void options.marker.line;
  if (options.overviewRulerOptions && !options.overviewRulerOptions.position) {
    options.overviewRulerOptions.position = 'full';
  }
  return record;
}
function registerDecoration(options) {
  const record = prepare(options);
  if (!record) return;
  return this._registerDecoration(record, record.marker, this);
}
export function decorationRegistration() { return registerDecoration; }
export function pinDecoration(owner, record) {
  let values = owners.get(owner);
  if (!values) owners.set(owner, values = new Set());
  values.add(record);
  records.set(record, { owner: new WeakRef(owner), events: [] });
  return new WeakRef(record);
}
export function decorationField(weak, name) { return weak.deref().options[name]; }
export function decorationFinish(weak) {
  const record = weak.deref();
  if (!record) return;
  const state = records.get(record);
  for (const event of state.events) event.clear();
  owners.get(state.owner.deref())?.delete(record);
}
export function decorationListener(weak, payload) {
  const entry = weak.deref();
  // A collected JS owner cannot observe callbacks. Native listeners hold no
  // strong edge back into the JS callback/options graph.
  if (entry) entry.listener.call(entry.thisArgs, payload);
}
function eventFunction(subscribe, entries) {
  return (listener, thisArgs, container) => {
    const entry = { listener, thisArgs };
    const native = subscribe(new WeakRef(entry));
    if (native === undefined) return { dispose() {} };
    entries.add(entry);
    const result = { dispose() { native.dispose(); entries.delete(entry); } };
    if (container) {
      if (Array.isArray(container)) container.push(result);
      else container.add(result);
    }
    return result;
  };
}
export function decorateRecord(record, dispose, element, setElement, isDisposed, render, disposed) {
  const state = records.get(record);
  const renders = new Set(), disposals = new Set();
  state.events.push(renders, disposals);
  record.onRender = eventFunction(render, renders);
  record.onDispose = eventFunction(disposed, disposals);
  // Callbacks own native capabilities. Native capabilities only weakly refer
  // to this record, so even options/listener cycles are collected by JS GC.
  Object.defineProperty(record, 'element', { enumerable: true, configurable: true,
    get: element, set: setElement });
  Object.defineProperty(record, 'isDisposed', { enumerable: true, configurable: true,
    get: isDisposed });
  record.dispose = function () {
    if (this !== record) throw new TypeError('Invalid decoration receiver');
    dispose();
  };
  return record;
}
