// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
// The same assertions run against unchanged upstream and the Rust DOM adapter.
export const fastCases = [
  'dimensions', 'failed_assignment', 'failed_conversion', 'class_reentry',
  'class_toggle', 'attributes', 'raw_utf16', 'layer_hint',
];
export function runFastCase(name, factory) {
  let assertions = 0;
  const equal = (actual, expected) => {
    assertions++;
    if (!Object.is(actual, expected)) throw Error(`${name}: expected ${String(expected)}, got ${String(actual)}`);
  };
  const throws = (callback, expected) => {
    assertions++;
    try { callback(); } catch (error) {
      if (error !== expected) throw Error(`${name}: thrown value identity changed`);
      return;
    }
    throw Error(`${name}: expected exception`);
  };
  const node = document.createElement('div');
  const style = node.style;
  const writes = [];
  const proxy = new Proxy(style, {
    get(target, key) { const value = Reflect.get(target, key, target); return typeof value === 'function' ? value.bind(target) : value; },
    set(target, key, value) { writes.push([key, value]); return Reflect.set(target, key, value, target); },
  });
  Object.defineProperty(node, 'style', {configurable: true, get() { return proxy; }});
  const fast = factory(node);
  try {
    switch (name) {
      case 'dimensions': {
        for (const key of ['Width', 'Height', 'Top', 'Left', 'Bottom', 'Right']) {
          const start = writes.length;
          fast[`set${key}`]('');
          equal(writes.length, start);
          fast[`set${key}`](-0);
          fast[`set${key}`]('0px');
          equal(writes.length, start + 1);
          fast[`set${key}`](-2.5);
          equal(writes.at(-1)[1], '-2.5px');
          fast[`set${key}`](NaN);
          fast[`set${key}`](NaN);
          equal(writes.at(-1)[1], 'NaNpx');
          equal(writes.length, start + 3);
          fast[`set${key}`](Infinity);
          equal(writes.at(-1)[1], 'Infinitypx');
        }
        fast.setPosition('');
        equal(writes.length, 24);
        fast.setPosition('absolute');
        fast.setPosition('absolute');
        equal(writes.length, 25);
        equal(style.position, 'absolute');
        fast.setContain('none');
        equal(writes.length, 25);
        fast.setContain('strict');
        fast.setContain('strict');
        equal(writes.length, 26);
        break;
      }
      case 'failed_assignment': {
        const error = {case: name};
        let gets = 0;
        Object.defineProperty(node, 'style', {configurable: true, get() { gets++; throw error; }});
        throws(() => fast.setWidth(12), error);
        fast.setWidth(12);
        equal(gets, 1);
        Object.defineProperty(node, 'style', {configurable: true, get() { return proxy; }});
        fast.setWidth(13);
        equal(style.width, '13px');
        equal(writes.length, 1);
        break;
      }
      case 'failed_conversion': {
        const error = Symbol(name);
        let conversions = 0;
        const value = {[Symbol.toPrimitive](hint) { conversions++; equal(hint, 'string'); throw error; }};
        throws(() => fast.setHeight(value), error);
        fast.setHeight(value);
        equal(conversions, 1);
        equal(style.height, '');
        equal(writes.length, 1);
        break;
      }
      case 'class_reentry': {
        let current = '';
        const assigned = [];
        Object.defineProperty(node, 'className', {configurable: true,
          get() { return current; }, set(value) { assigned.push(value); current = String(value); }});
        const value = {[Symbol.toPrimitive]() { fast.setClassName('nested'); return 'outer'; }};
        fast.setClassName(value);
        equal(assigned.length, 2);
        equal(current, 'outer');
        fast.setClassName('nested');
        equal(assigned.length, 2);
        equal(current, 'outer');
        break;
      }
      case 'class_toggle': {
        fast.setClassName('one');
        fast.toggleClassName('two', true);
        equal(node.className, 'one two');
        fast.setClassName('one two');
        fast.toggleClassName('two');
        equal(node.className, 'one');
        fast.toggleClassName('three', false);
        equal(node.className, 'one');
        let calls = 0;
        const force = {[Symbol.toPrimitive]() { calls++; return false; }};
        fast.toggleClassName('three', force);
        equal(node.className, 'one three');
        equal(calls, 0);
        fast.setClassName('one three');
        equal(node.className, 'one three');
        break;
      }
      case 'attributes': {
        const assigned = [];
        Object.defineProperty(node, 'setAttribute', {configurable: true, writable: true,
          value(name, value) { assigned.push([name, value]); }});
        const value = {[Symbol.toPrimitive]() { throw Error('eager conversion'); }};
        fast.setAttribute('data-value', value);
        fast.setAttribute('data-value', value);
        equal(assigned.length, 2);
        equal(assigned[0][1], value);
        const error = 42;
        node.setAttribute = () => { throw error; };
        throws(() => fast.setAttribute('data-value', value), error);
        break;
      }
      case 'raw_utf16': {
        const value = '\ud800x\udfff';
        fast.setPosition(value);
        equal(writes[0][1], value);
        let coercions = 0;
        const raw = {[Symbol.toPrimitive](hint) { coercions++; equal(hint, 'string'); return value; }};
        fast.setWidth(raw);
        fast.setWidth(raw);
        equal(writes[1][1], raw);
        equal(coercions, 1);
        fast.setWidth(value);
        equal(writes.length, 3);
        break;
      }
      case 'layer_hint': {
        fast.setLayerHinting(false);
        equal(writes.length, 0);
        fast.setLayerHinting(true);
        fast.setLayerHinting(true);
        equal(writes.length, 1);
        equal(writes[0][1], 'translate3d(0px, 0px, 0px)');
        fast.setLayerHinting(false);
        equal(writes.length, 2);
        equal(writes[1][1], '');
        break;
      }
      default: throw Error(`Unknown FastDomNode case ${name}`);
    }
    return assertions;
  } finally {
    delete node.style;
    node.remove();
  }
}
