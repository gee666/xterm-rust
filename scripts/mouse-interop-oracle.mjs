// Copyright (c) 2019-2026 The xterm.js authors. All rights reserved. MIT license.
// Source-only raw interop probes. No Rust or full binding parity is claimed.
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { createEvidenceRun, persistEvidence } from './upstream-baseline-utils.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const attempt = createEvidenceRun(path.join(root, 'results/mouse-interop'), { kind: 'original-mouse-interop' });
const report = { node: process.version, platform: process.platform, arch: process.arch,
  originalSuitesExecuted: 0, rustCasesExecuted: 0,
  scope: 'Additional unchanged-source observations only. Task171 must implement and compare actual Rust/WASM behavior.',
  checks: [], observations: {}, status: 'incomplete' };
function check(name, callback) {
  callback();
  report.checks.push({ name, status: 'passed' });
}
function units(text) { return Array.from({ length: text.length }, (_, i) => text.charCodeAt(i)); }
function value(value, refs = {}) {
  for (const [name, reference] of Object.entries(refs)) if (value === reference) return { type: typeof value, reference: name };
  if (typeof value === 'string') return { type: 'string', utf16: units(value) };
  if (typeof value === 'number') return { type: 'number', text: Object.is(value, -0) ? '-0' : String(value) };
  if (typeof value === 'bigint') return { type: 'bigint', text: String(value) };
  if (typeof value === 'boolean' || value === null) return { type: value === null ? 'null' : 'boolean', value };
  if (typeof value === 'undefined') return { type: 'undefined' };
  return { type: typeof value };
}
function outcome(callback, refs = {}) {
  try { return { status: 'returned', value: value(callback(), refs) }; }
  catch (error) {
    return { status: 'threw', value: value(error, refs), name: error?.name,
      messageUtf16: typeof error?.message === 'string' ? units(error.message) : undefined };
  }
}
function event() { return { col: 1, row: 2, x: 3, y: 4, button: 0, action: 1, ctrl: false, alt: false, shift: false }; }

try {
  const { loadMouseReference } = await import('./mouse-reference-module.mjs');
  const reference = loadMouseReference();
  const { MouseStateService } = reference;
  report.reference = { sourceHashes: reference.sourceHashes, moduleSha256: reference.moduleSha256, esbuild: reference.esbuild };
  const service = new MouseStateService();
  report.observations.serviceOwnKeys = Object.keys(service);
  report.observations.servicePrototype = Object.entries(Object.getOwnPropertyDescriptors(Object.getPrototypeOf(service)))
    .map(([name, descriptor]) => ({ name, method: typeof descriptor.value === 'function', getter: !!descriptor.get,
      setter: !!descriptor.set, enumerable: descriptor.enumerable, configurable: descriptor.configurable }));

  check('wheel receives original event and stable real service receiver', () => {
    const input = event();
    const calls = [];
    service.setCustomWheelEventHandler(function(actual) {
      calls.push({ receiver: this, actual, protocol: this.activeProtocol, encoding: this.activeEncoding,
        ownKeys: Object.keys(this), constructorName: this.constructor.name });
      actual.extra = input;
      return true;
    });
    assert.equal(service.allowCustomWheelEvent(input), true);
    service.activeProtocol = 'ANY'; service.activeEncoding = 'SGR';
    assert.equal(service.allowCustomWheelEvent(input), true);
    assert.equal(calls[0].receiver, service); assert.equal(calls[1].receiver, service);
    assert.equal(calls[0].actual, input); assert.equal(calls[1].actual, input);
    assert.equal(input.extra, input);
    assert.deepEqual(calls.map(call => [call.protocol, call.encoding]), [['NONE', 'DEFAULT'], ['ANY', 'SGR']]);
    report.observations.wheelReceiver = { sameService: true, sameEvent: true, extraIdentity: true,
      states: calls.map(call => [call.protocol, call.encoding]), ownKeys: calls[0].ownKeys,
      constructorName: calls[0].constructorName };
  });

  check('only literal boolean false vetoes wheel', () => {
    const cases = [['undefined', undefined], ['null', null], ['false', false], ['true', true], ['zero', 0],
      ['negativeZero', -0], ['one', 1], ['NaN', NaN], ['empty', ''], ['text', 'x'], ['object', {}],
      ['array', []], ['symbol', Symbol('result')], ['bigintZero', 0n], ['boxedFalse', new Boolean(false)]];
    report.observations.wheelResults = cases.map(([name, result]) => {
      service.setCustomWheelEventHandler(() => result);
      const allowed = service.allowCustomWheelEvent(event());
      assert.equal(allowed, name !== 'false');
      return { name, result: value(result), allowed };
    });
  });

  check('direct callback call ignores poisoned Function.call property', () => {
    let poisonReads = 0;
    const handler = function() { assert.equal(this, service); return true; };
    Object.defineProperty(handler, 'call', { get() { poisonReads++; throw new Error('must not read call'); } });
    service.setCustomWheelEventHandler(handler);
    assert.equal(service.allowCustomWheelEvent(event()), true);
    assert.equal(poisonReads, 0);
    report.observations.poisonedCallReads = poisonReads;
  });

  check('Event callback deliberately observes Function.call unlike wheel callbacks', () => {
    const instance = new MouseStateService();
    const sentinel = new Error('event call getter');
    let reads = 0; let bodies = 0;
    const handler = function() { bodies++; };
    Object.defineProperty(handler, 'call', { get() { reads++; throw sentinel; } });
    instance.onProtocolChange(handler);
    assert.throws(() => { instance.activeProtocol = 'NONE'; }, error => error === sentinel);
    assert.equal(reads, 1); assert.equal(bodies, 0);
    assert.equal(instance.activeProtocol, 'NONE');
    report.observations.eventCallProperty = { reads, bodies, committedProtocol: instance.activeProtocol };
  });

  check('every thrown value retains identity instead of false or empty fallback', () => {
    const sentinels = [new Error('original'), {}, 7, '\ud800', undefined, null, false, Symbol('throw')];
    report.observations.callbackThrows = sentinels.map(sentinel => {
      service.setCustomWheelEventHandler(() => { throw sentinel; });
      let caught = false;
      try { service.allowCustomWheelEvent(event()); }
      catch (error) { caught = true; assert.equal(error, sentinel); }
      assert.equal(caught, true);
      return { thrown: value(sentinel), identityRetained: true };
    });
  });

  check('restriction and encoding have distinct actual method receivers', () => {
    const input = event();
    let protocolReceiver; let encodingReceiver;
    const protocol = { events: 1, restrict(actual) { protocolReceiver = this; actual.extra = input; return true; } };
    const encoding = function(actual) { encodingReceiver = this; assert.equal(actual, input); assert.equal(actual.extra, input); return '\ud800'; };
    service.addProtocol('CUSTOM', protocol); service.addEncoding('CUSTOM', encoding);
    service.activeProtocol = 'CUSTOM'; service.activeEncoding = 'CUSTOM';
    assert.equal(service.restrictMouseEvent(input), true);
    assert.equal(service.encodeMouseEvent(input), '\ud800');
    assert.equal(protocolReceiver, protocol); assert.equal(encodingReceiver, service._encodings);
    protocol.events = 0; assert.equal(service.areMouseEventsActive, false);
    report.observations.methodReceivers = { protocolIdentity: true, registryIdentity: true, eventIdentity: true, loneSurrogate: [0xd800] };
  });

  check('inherited registry names are not ordinary unknown-name failures', () => {
    const names = ['constructor', '__defineGetter__', '__defineSetter__', 'hasOwnProperty', '__lookupGetter__',
      '__lookupSetter__', 'isPrototypeOf', 'propertyIsEnumerable', 'toString', 'valueOf', '__proto__', 'toLocaleString', 'missing', '\ud800'];
    report.observations.inherited = names.map(name => {
      const instance = new MouseStateService();
      const input = event();
      const masks = [];
      instance.onProtocolChange(mask => masks.push(value(mask)));
      const protocol = outcome(() => { instance.activeProtocol = name; return instance.activeProtocol; });
      const active = outcome(() => instance.areMouseEventsActive);
      const restrict = outcome(() => instance.restrictMouseEvent(input));
      const encoding = outcome(() => { instance.activeEncoding = name; return instance.activeEncoding; });
      const encoded = outcome(() => instance.encodeMouseEvent(input), { event: input, registry: instance._encodings });
      if (!['missing', '\ud800'].includes(name)) {
        assert.equal(protocol.status, 'returned'); assert.equal(encoding.status, 'returned');
        assert.deepEqual(masks, [{ type: 'undefined' }]);
      } else { assert.equal(protocol.status, 'threw'); assert.equal(encoding.status, 'threw'); }
      return { nameUtf16: units(name), protocol, active, masks, restrict, encoding, encoded };
    });
  });

  check('numeric registry keys use Object.keys order', () => {
    const instance = new MouseStateService();
    const protocol = { events: 0, restrict: () => false };
    for (const name of ['10', '2', '0', '4294967294', '4294967295', '01']) instance.addProtocol(name, protocol);
    const keys = Object.keys(instance._protocols);
    assert.deepEqual(keys.slice(0, 4), ['0', '2', '10', '4294967294']);
    report.observations.numericKeys = keys;
  });

  check('__proto__ registration changes prototype rather than adding own key', () => {
    const instance = new MouseStateService();
    const protocol = { events: 1, restrict: () => true };
    instance.addProtocol('__proto__', protocol);
    assert.equal(Object.getPrototypeOf(instance._protocols), protocol);
    assert.equal(Object.keys(instance._protocols).includes('__proto__'), false);
    instance.activeProtocol = '__proto__'; assert.equal(instance.restrictMouseEvent(event()), true);
    let receiver;
    const encoder = function() { receiver = this; return 'ok'; };
    instance.addEncoding('__proto__', encoder);
    assert.equal(Object.getPrototypeOf(instance._encodings), encoder);
    assert.equal(Object.keys(instance._encodings).includes('__proto__'), false);
    instance.activeEncoding = '__proto__'; assert.equal(instance.encodeMouseEvent(event()), 'ok');
    assert.equal(receiver, instance._encodings);
    report.observations.protoRegistration = { ownProtocol: false, ownEncoding: false, liveProtocol: true, liveEncoding: true };
  });
  report.status = 'passed-source-only';
} catch (error) {
  report.status = 'failed'; report.error = String(error.stack || error); process.exitCode = 1;
} finally {
  persistEvidence(attempt, report);
}
console.log(JSON.stringify({ status: report.status, checks: report.checks.length, runId: attempt.id, error: report.error }));
