// Shared real-DOM probe. Both engines parse the same OSC8 before run().
const screen = root.querySelector('.xterm-screen');
const log = [];
const handler = Object.fromEntries(['hover', 'leave', 'activate'].map(name => [name,
  function (event, text, range) {
    log.push([name, event.type, text, range.start.x, range.end.x]);
  }
]));
const snapshots = [];
let x, y;
const send = (element, type, bubbles = false) => element.dispatchEvent(new MouseEvent(type,
  { clientX: x, clientY: y, bubbles }));
const snapshot = label => snapshots.push({ label,
  rootPointer: root.classList.contains('xterm-cursor-pointer'),
  screenPointer: screen.classList.contains('xterm-cursor-pointer'),
  log: log.map(entry => [...entry]) });
return {
  handler,
  run() {
    // Distinguish root and screen origins, and require screen padding subtraction.
    root.style.padding = '31px 29px';
    screen.style.padding = '23px 19px';
    const rect = screen.getBoundingClientRect();
    x = rect.left + 19 + 5.5 * width;
    y = rect.top + 23 + 0.5 * height;
    send(root, 'mousemove');
    snapshot('root movement ignored');
    send(screen, 'mousemove');
    snapshot('screen hover');
    send(root, 'mousedown');
    send(root, 'mouseup');
    snapshot('root click ignored');
    send(screen, 'mousedown', true);
    send(screen, 'mouseup', true);
    snapshot('screen click');
    send(root, 'mouseleave');
    snapshot('root leave ignored');
    send(screen, 'mouseleave');
    snapshot('nonbubbling screen leave');
    return snapshots;
  },
  afterDispose() {
    send(screen, 'mousemove');
    send(screen, 'mousedown');
    send(screen, 'mouseup');
    send(screen, 'mouseleave');
    return log;
  }
};
