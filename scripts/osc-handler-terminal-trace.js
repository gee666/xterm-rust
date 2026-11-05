// Real DOM events, shared by the original and Rust-backed Terminal tests.
const log = [];
const keys = ['allowNonHttpProtocols', 'activate', 'hover', 'leave'];
const saved = new Map(keys.map(key => [key, Object.getOwnPropertyDescriptor(Boolean.prototype, key)]));
const savedConfirm = Object.getOwnPropertyDescriptor(window, 'confirm');
const screen = root.querySelector('.xterm-screen');
const call = type => screen.dispatchEvent(new MouseEvent(type, { clientX: x, clientY: y, bubbles: true }));
// The port currently applies the cursor class to its root, source to its screen.
// This probe checks the visible state, not that separate DOM-placement contract.
const pointer = () => root.classList.contains('xterm-cursor-pointer') ||
  screen.classList.contains('xterm-cursor-pointer');
try {
  Object.defineProperty(Boolean.prototype, 'allowNonHttpProtocols', { configurable: true, get() {
    'use strict';
    log.push(['policy', typeof this, this === false]);
    return true;
  } });
  Object.defineProperty(Boolean.prototype, 'activate', { configurable: true, get() {
    throw new Error('Falsey handler must use default activation');
  } });
  for (const name of ['hover', 'leave']) {
    Object.defineProperty(Boolean.prototype, name, { configurable: true, value(event, text, range) {
      'use strict';
      log.push([name, this === false, event.type, text, range.start.x, range.end.x]);
    } });
  }
  Object.defineProperty(window, 'confirm', { configurable: true, value(text) {
    log.push(['confirm', text]);
    return false;
  } });
  call('mousemove');
  const pointerOn = pointer();
  call('mousedown');
  call('mouseup');
  call('mouseleave');
  return { pointerOn, pointerOff: !pointer(), log };
} finally {
  for (const [key, descriptor] of saved) {
    if (descriptor) Object.defineProperty(Boolean.prototype, key, descriptor);
    else delete Boolean.prototype[key];
  }
  if (savedConfirm) Object.defineProperty(window, 'confirm', savedConfirm);
  else delete window.confirm;
}
