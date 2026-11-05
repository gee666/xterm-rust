// Source contract: common/input/WriteBuffer.ts. The clock fixes the queue's
// 12ms slicing decision, not the browser's animation-frame or timer delivery.
function controlledClock() {
  let now = 100;
  const performance = globalThis.performance;
  const original = Object.getOwnPropertyDescriptor(performance, 'now');
  Object.defineProperty(performance, 'now', { configurable: true, value: () => now });
  return {
    advance: milliseconds => { now += milliseconds; },
    restore: () => {
      if (original) Object.defineProperty(performance, 'now', original);
      else delete performance.now;
    },
  };
}

export async function writerReentryCase(terminal, exceedBudget, render = false) {
  const events = [];
  const cursor = () => terminal.buffer.active.cursorX;
  let parent;
  if (render) {
    parent = document.createElement('div');
    document.body.append(parent);
    await new Promise(resolve => {
      const subscription = terminal.onRender(() => { subscription.dispose(); resolve(); });
      terminal.open(parent);
      terminal.refresh(0, terminal.rows - 1);
    });
  }
  let rendered;
  let renderSubscription;
  const firstRender = render ? new Promise(resolve => {
    renderSubscription = terminal.onRender(() => {
      renderSubscription.dispose();
      rendered = cursor();
      resolve();
    });
  }) : undefined;
  const clock = controlledClock();
  let entered = false;
  const subscriptions = [
    terminal.onData(data => {
      events.push(['data', data, cursor()]);
      if (entered) return;
      entered = true;
      terminal.options.disableStdin = true;
      terminal.input('MUST_NOT_EMIT', true);
      terminal.options.disableStdin = false;
      terminal.write('R', () => events.push(['reentry-complete', cursor()]));
      if (exceedBudget) clock.advance(13);
    }),
    terminal.onCursorMove(() => events.push(['cursor', cursor()])),
  ];
  try {
    await new Promise(resolve => {
      subscriptions.push(terminal.onWriteParsed(() => {
        events.push(['parsed', cursor()]);
        if (cursor() === 3) resolve();
      }));
      terminal.write('P\x1b[?1h\x1b[?1$pQ\x1b[4h\x1b[4$p',
        () => events.push(['outer-complete', cursor()]));
    });
    if (firstRender) await firstRender;
    return { events, text: terminal.buffer.active.getLine(0).translateToString(true),
      ...(render ? { firstRenderedCursor: rendered } : {}) };
  } finally {
    if (renderSubscription) renderSubscription.dispose();
    for (const subscription of subscriptions) subscription.dispose();
    clock.restore();
    terminal.dispose();
    if (parent) parent.remove();
  }
}

export async function writerCompletionCase(terminal) {
  const events = [];
  const cursor = () => terminal.buffer.active.cursorX;
  const clock = controlledClock();
  try {
    await new Promise(resolve => {
      const subscription = terminal.onWriteParsed(() => {
        events.push(['parsed', cursor()]);
        if (cursor() === 2) {
          subscription.dispose();
          resolve();
        }
      });
      terminal.write('A', () => {
        events.push(['first', cursor()]);
        terminal.write('B', () => events.push(['second', cursor()]));
      });
    });
    return { events, text: terminal.buffer.active.getLine(0).translateToString(true) };
  } finally { clock.restore(); terminal.dispose(); }
}

export async function writerDisposalCase(terminal) {
  const events = [];
  terminal.onWriteParsed(() => events.push('parsed'));
  terminal.write('A', () => events.push('completion'));
  terminal.dispose();
  // A sentinel scheduled after dispose observes the cancelled write's turn.
  await new Promise(resolve => setTimeout(resolve, 0));
  return events;
}
