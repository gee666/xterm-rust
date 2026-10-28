// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
export const visibilityCases = ['modes_and_timing', 'replacement', 'dispose', 'host_errors', 'class_errors'];
export function runVisibilityCase(name, factory) {
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
  const original = {setTimeout: window.setTimeout, clearTimeout: window.clearTimeout};
  const pending = new Map();
  let next = 0, requestError, cancelError;
  window.setTimeout = (fn, delay) => {
    equal(delay, 0);
    if (requestError !== undefined) throw requestError;
    pending.set(++next, fn);
    return next;
  };
  window.clearTimeout = id => {
    if (cancelError !== undefined) throw cancelError;
    pending.delete(id);
  };
  const tick = () => {
    equal(pending.size, 1);
    const [id, fn] = pending.entries().next().value;
    pending.delete(id);
    fn();
  };
  const node = document.createElement('div');
  let controller;
  try {
    controller = factory(name === 'replacement' ? 3 : 1, node);
    equal(node.className, 'hidden');
    if (name === 'modes_and_timing') {
      controller.setShouldBeVisible(true);
      equal(pending.size, 0);
      controller.setIsNeeded(true);
      equal(node.className, 'hidden');
      tick();
      equal(node.className, 'shown');
      controller.setShouldBeVisible(false);
      equal(node.className, 'hidden xterm-fade');
      controller.setVisibility(3);
      tick();
      equal(node.className, 'shown');
      controller.setIsNeeded(false);
      equal(node.className, 'hidden');
      controller.setIsNeeded(true);
      controller.setVisibility(2);
      equal(pending.size, 0);
      equal(node.className, 'hidden xterm-fade');
      controller.setVisibility('2');
      equal(pending.size, 0);
      controller.setShouldBeVisible(true);
      tick();
      equal(node.className, 'shown');
    } else if (name === 'replacement') {
      controller.setIsNeeded(true);
      const replacement = document.createElement('div');
      controller.setDomNode(replacement);
      tick();
      equal(node.className, 'hidden');
      equal(replacement.className, 'shown');
      const third = document.createElement('div');
      controller.setDomNode(third);
      controller.ensureVisibility();
      equal(third.className, 'hidden');
      equal(pending.size, 0);
    } else if (name === 'dispose') {
      controller.setIsNeeded(true);
      controller.setShouldBeVisible(true);
      equal(pending.size, 1);
      controller.dispose();
      equal(pending.size, 0);
      controller.setShouldBeVisible(false);
      let error;
      try { controller.setShouldBeVisible(true); } catch (e) { error = e; }
      equal(error?.message, 'Calling setIfNotSet on a disposed TimeoutTimer');
      controller.ensureVisibility();
      equal(pending.size, 0);
      controller.setVisibility(2);
      equal(node.className, 'hidden xterm-fade');
    } else if (name === 'host_errors') {
      controller.setIsNeeded(true);
      requestError = {request: true};
      throws(() => controller.setShouldBeVisible(true), requestError);
      requestError = undefined;
      controller.ensureVisibility();
      equal(pending.size, 0);
      controller.setShouldBeVisible(false);
      controller.setShouldBeVisible(true);
      cancelError = Symbol('cancel');
      throws(() => controller.setIsNeeded(false), cancelError);
      equal(pending.size, 1);
      cancelError = undefined;
      controller.ensureVisibility();
      equal(pending.size, 0);
      equal(node.className, 'hidden');
    } else if (name === 'class_errors') {
      controller.setIsNeeded(true);
      controller.setShouldBeVisible(true);
      const error = {className: true};
      Object.defineProperty(node, 'className', {configurable: true, get() { return 'intercepted'; }, set() { throw error; }});
      throws(tick, error);
      equal(pending.size, 0);
      delete node.className;
      controller.ensureVisibility();
      equal(pending.size, 0);
      controller.setShouldBeVisible(false);
      equal(node.className, 'hidden xterm-fade');
      controller.setShouldBeVisible(true);
      tick();
      equal(node.className, 'shown');
    } else throw Error(`Unknown visibility case ${name}`);
    return assertions;
  } finally {
    requestError = undefined;
    cancelError = undefined;
    try { controller?.dispose(); }
    finally { Object.assign(window, original); node.remove(); }
  }
}
