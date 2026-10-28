// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
export const widgetCases = ['kinds_and_cleanup', 'recursive_disposal', 'adopted_window', 'callback_error'];
export function runWidgetCase(name, factory) {
  let assertions = 0;
  const equal = (actual, expected) => {
    assertions++;
    if (!Object.is(actual, expected)) throw Error(`${name}: expected ${String(expected)}, got ${String(actual)}`);
  };
  const node = document.createElement('div');
  const cleanup = [];
  const fire = (type, extra = {}) => {
    const event = new MouseEvent(type, {view: window, bubbles: true, cancelable: true,
      clientX: 42, clientY: 63, button: 1, buttons: 4, detail: 0, ctrlKey: true, ...extra});
    node.dispatchEvent(event);
    return event;
  };
  const listen = (type, callback) => {
    const dispose = factory(node, type, callback);
    cleanup.push(dispose);
    return dispose;
  };
  document.body.appendChild(node);
  try {
    if (name === 'kinds_and_cleanup') {
      const calls = [];
      for (const type of ['click', 'mouseover', 'mouseleave']) {
        listen(type, e => {
          calls.push(type);
          equal(e.browserEvent.type, type);
          equal(e.target, node);
          // Chromium normalizes button on mouseover/mouseleave. Match the
          // actual delivered MouseEvent, not only its constructor options.
          equal(e.leftButton, e.browserEvent.button === 0);
          equal(e.middleButton, e.browserEvent.button === 1);
          equal(e.rightButton, e.browserEvent.button === 2);
          equal(e.buttons, 4);
          equal(e.detail, 0);
          equal(e.ctrlKey, true);
          equal(e.shiftKey, false);
          equal(e.posx, e.browserEvent.pageX);
          equal(e.posy, e.browserEvent.pageY);
          equal(typeof e.timestamp, 'number');
          e.preventDefault();
          e.stopPropagation();
        });
        equal(fire(type).defaultPrevented, true);
      }
      equal(calls.join(','), 'click,mouseover,mouseleave');
      for (const dispose of cleanup) dispose();
      for (const type of ['click', 'mouseover', 'mouseleave']) fire(type);
      equal(calls.length, 3);
    } else if (name === 'recursive_disposal') {
      const calls = [];
      let dispose;
      dispose = listen('click', () => {
        calls.push('start');
        if (calls.length === 1) fire('click');
        else dispose();
        calls.push('end');
      });
      fire('click');
      fire('click');
      equal(calls.join(','), 'start,start,end,end');
    } else if (name === 'adopted_window') {
      const frame = document.createElement('iframe');
      frame.style.cssText = 'position:fixed;left:17px;top:23px;border:0;width:100px;height:100px';
      document.body.appendChild(frame);
      try {
        let current;
        listen('click', e => { current = e; });
        frame.contentDocument.body.appendChild(frame.contentDocument.adoptNode(node));
        const bounds = frame.getBoundingClientRect();
        const event = fire('click');
        equal(current.target, node);
        equal(current.posx, event.pageX - bounds.left - window.scrollX);
        equal(current.posy, event.pageY - bounds.top - window.scrollY);
      } finally { frame.remove(); }
    } else if (name === 'callback_error') {
      const thrown = {widget: name};
      const errors = [];
      const capture = event => { errors.push(event.error); event.preventDefault(); };
      window.addEventListener('error', capture);
      try {
        let later = 0;
        const dispose = listen('click', () => { throw thrown; });
        node.addEventListener('click', () => { later++; });
        fire('click');
        equal(errors.length, 1);
        equal(errors[0], thrown);
        equal(later, 1);
        dispose();
        fire('click');
        equal(errors.length, 1);
        equal(later, 2);
      } finally { window.removeEventListener('error', capture); }
    } else throw Error(`Unknown Widget case ${name}`);
    return assertions;
  } finally {
    for (const dispose of cleanup) dispose();
    node.remove();
  }
}
