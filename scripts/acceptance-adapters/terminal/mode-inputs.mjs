import assert from 'node:assert/strict';
import { capturePixels } from '../../browser-capture/host.mjs';
export const caseIds = ['vt-28', 'vt-29', 'vt-30', 'vt-31', 'vt-34', 'vt-40', 'vt-41'];
const clone = value => JSON.parse(JSON.stringify(value));
const line = (state, buffer, row) => state[buffer].lines[row]?.cells.map(cell =>
  cell.chars.length ? String.fromCharCode(...cell.chars) : cell.width ? ' ' : '').join('') ?? '';

export async function exerciseModeCase(page, host, implementation, engine, caseId) {
  const frames = []; const pixels = []; const inputs = []; let assertions = 0;
  const check = (value, message) => { assertions++; assert.ok(value, message); };
  const same = (actual, expected, message) => { assertions++; assert.deepEqual(actual, expected, message); };
  const write = text => {
    inputs.push({ kind: 'write', units: Array.from({ length: text.length }, (_, index) => text.charCodeAt(index)) });
    return page.evaluate(text => window.captureSession.write(text), text);
  };
  const call = (method, args = []) => {
    inputs.push({ kind: 'api', method, args: clone(args) });
    return page.evaluate(({ method, args }) => {
    const terminal = window.captureSession.terminal;
    if (typeof terminal[method] !== 'function') throw new Error(`Missing real ${method} API`);
    return terminal[method](...args);
    }, { method, args });
  };
  const press = key => { inputs.push({ kind: 'key', key }); return page.keyboard.press(key); };
  async function frame(label) {
    const value = await page.evaluate(label => window.captureSession.capture(label), label);
    frames.push(value);
    const image = await capturePixels(page, host, `${engine}-${implementation}-${label}`);
    pixels.push({ label, ...image });
    check(value.state.schema === 1, 'Actual full-state acceptance schema required');
    check(value.properties.rootCount === 1, 'Actions must target one visible terminal');
    return value;
  }
  if (caseId === 'vt-28' || caseId === 'vt-29') {
    const mode = caseId === 'vt-28' ? 47 : 1047;
    await write('MAIN\x1b[31m red界\x1b[0m'); await frame('normal');
    await write(`\x1b[42m\x1b[?${mode}hALT\x1b[34m blue\x1b[0m`);
    const first = await frame('alternate');
    same(first.state.active, 'alternate'); check(line(first.state, 'normal', 0).includes('MAIN'), 'Normal cells survive');
    check(line(first.state, 'alternate', 0).includes('ALT'), 'Real alternate output must be visible');
    const changes = first.events.filter(event => event.type === 'buffer').length;
    await write(`\x1b[?${mode}h`);
    const repeated = await frame('alternate-repeat');
    same(repeated.events.filter(event => event.type === 'buffer').length, changes, 'Repeated activation must not invent buffer change');
    await write(`\x1b[?${mode}lBACK`);
    const normal = await frame('normal-return');
    same(normal.state.active, 'normal'); check(line(normal.state, 'normal', 0).includes('BACK'), 'Return writes affect actual normal cells');
    same(normal.state.alternate.lines.length, 0, 'Source normal activation clears alternate content');
    await write(`\x1b[?${mode}l`); await frame('normal-repeat');
    await write(`\x1b[?${mode}hNEW`); const next = await frame('alternate-new');
    check(!line(next.state, 'alternate', 0).includes('ALT'), 'Old alternate text must not revive');
    check(next.events.some(event => event.type === 'internal' && event.value.name === 'syncScrollBar'), 'Actual scrollbar request must be captured');
  } else if (caseId === 'vt-30') {
    await write('\x1b[3;5H\x1b[31m'); const original = await frame('cursor-original');
    await write('\x1b[?1048h'); await frame('cursor-saved');
    await write('\x1b[?6h\x1b[?7l\x1b[1;1H\x1b[32mMOVED\x1b(0'); await frame('cursor-mutated');
    await write('\x1b[?1048l'); const restored = await frame('cursor-restored');
    same(restored.state.active, 'normal');
    same([restored.state.normal.cursorX, restored.state.normal.cursorY], [original.state.normal.cursorX, original.state.normal.cursorY]);
    same(restored.state.attributes.fg, original.state.attributes.fg);
    same(restored.state.modes.origin, original.state.modes.origin);
    same(restored.state.modes.wraparound, original.state.modes.wraparound);
    same(restored.events.filter(event => event.type === 'buffer').length, 0, '1048 must not invent a buffer change');
  } else if (caseId === 'vt-31') {
    await write('MAIN\x1b[2;4H\x1b[31mDATA');
    await call('select', [0, 0, 4]); const normal = await frame('selected-normal');
    same(normal.properties.selection, 'MAIN');
    await write('\x1b[42m\x1b[?1049h'); const blank = await frame('alternate-erased');
    same(blank.state.active, 'alternate'); same(blank.properties.selection, '', 'Buffer switch clears the actual selection');
    same(blank.state.alternate.lines[0].cells[0].bg & 255, 2, 'BCE uses the current green background');
    await write('\x1b[4;2H\x1b[34mALT\x1b[s'); await frame('alternate-cursor-saved');
    await write('\x1b[?1049l'); const returned = await frame('normal-restored');
    same([returned.state.normal.cursorX, returned.state.normal.cursorY], [normal.state.normal.cursorX, normal.state.normal.cursorY]);
    await write('\x1b[?1049h\x1b[uMORE'); const next = await frame('alternate-cursor-restored');
    same(next.state.alternate.lines[3].cells[4].chars, [77], 'Alt saved cursor must survive clear/re-entry');
    same(next.state.attributes.fg & 255, 4, 'Alt ANSI restore retains blue attributes');
    await write('\x1b[?1049h'); await frame('alternate-repeat');
    check(next.events.some(event => event.type === 'selection'), 'Real selection callbacks required');
  } else if (caseId === 'vt-34') {
    await page.evaluate(() => {
      let entered = false; window.modeReentryDone = false;
      const session = window.captureSession; const terminal = session.terminal;
      window.modeDataHandle = terminal.onData(data => {
        if (entered || !data.endsWith('$y')) return;
        entered = true;
        session.recordBusiness({ phase: 'before-reentry', data });
        terminal.options.disableStdin = true; terminal.input('MUST_NOT_EMIT', true);
        terminal.options.disableStdin = false;
        terminal.write('R', () => { window.modeReentryDone = true; session.recordBusiness({ phase: 'queued-write-complete' }); });
      });
    });
    await write('P\x1b[?1h\x1b[?1$pQ\x1b[4h\x1b[4$p');
    await page.waitForFunction(() => window.modeReentryDone === true);
    const reentry = await frame('query-reentry');
    check(line(reentry.state, 'normal', 0).startsWith('PQR'), 'Suffix must precede reentrant queued display write');
    check(reentry.events.some(event => event.type === 'data' && event.value === '\x1b[?1;1$y'), 'Exact DEC query reply');
    check(reentry.events.some(event => event.type === 'data' && event.value === '\x1b[4;1$y'), 'Exact ANSI query reply');
    check(reentry.events.every(event => event.type !== 'data' || event.value !== 'MUST_NOT_EMIT'), 'disableStdin vetoes reentrant input');
    const previous = reentry.events.filter(event => event.type === 'data').length;
    await page.evaluate(() => window.captureSession.terminal.options.disableStdin = true);
    await write('\x1b[?1l\x1b[?1$pBLOCKED'); const disabled = await frame('query-disabled');
    same(disabled.events.filter(event => event.type === 'data').length, previous, 'Disabled output must not invent query replies');
    same(disabled.state.modes.applicationCursorKeys, false, 'Incoming state still parses while stdin disabled');
    await page.evaluate(() => window.captureSession.terminal.options.disableStdin = false);
    await write('\x1b[?1$p'); await frame('query-enabled');
    await page.evaluate(() => window.modeDataHandle.dispose());
  } else if (caseId === 'vt-40') {
    await page.evaluate(() => window.captureSession.terminal.options.vtExtensions = { kittyKeyboard: true });
    await write('\x1b[>8u\x1b[?u'); await page.locator('.xterm-helper-textarea').focus();
    await press('a'); const main = await frame('kitty-main');
    same(main.state.kitty.flags, 8); check(main.events.some(event => event.type === 'data' && event.value === '\x1b[?8u'), 'Negotiated query reply');
    check(main.events.some(event => event.type === 'data' && event.value.startsWith('\x1b[97')), 'Real key event must use CSI-u');
    await write('\x1b[?1049h\x1b[?u'); await press('a'); const alternate = await frame('kitty-alt-default');
    same(alternate.state.kitty.flags, 0); check(alternate.events.some(event => event.type === 'data' && event.value === 'a'), 'Alt default must retain legacy key output');
    await write('\x1b[>8u\x1b[?1049h\x1b[?u'); await press('a'); await frame('kitty-alt-repeat');
    await write('\x1b[<99u\x1b[?u'); await press('a'); const popped = await frame('kitty-pop-empty');
    same(popped.state.kitty.flags, 0);
    await write('\x1b[?1049l\x1b[?u'); await press('a'); await frame('kitty-normal-return');
    await page.evaluate(() => window.captureSession.terminal.options.vtExtensions = { kittyKeyboard: false });
    await write('\x1b[=31u\x1b[?u'); await press('a'); await frame('kitty-disabled');
  } else if (caseId === 'vt-41') {
    await write('pointer test');
    const screen = await page.locator('.xterm-screen').boundingBox();
    if (!screen) throw new Error('Actual mouse screen unavailable');
    let position = 0;
    async function pointer() {
      const x = screen.x + 2 + position++ * Math.max(1, screen.width / 24);
      const y = screen.y + Math.max(1, screen.height / 12);
      inputs.push({ kind: 'pointer', start: [x, y], end: [x + screen.width / 24, y] });
      await page.mouse.move(x, y); await page.mouse.down();
      await page.mouse.move(x + screen.width / 24, y); await page.mouse.up();
    }
    for (const protocol of [9, 1000, 1002, 1003]) {
      await write(`\x1b[?${protocol}h`); await pointer(); await frame(`pointer-${protocol}`);
    }
    await write('\x1b[?1006h'); await pointer(); const sgr = await frame('pointer-sgr');
    same(sgr.state.modes.mouseEncoding, 'SGR');
    await write('\x1b[?1016h'); await pointer(); const pixel = await frame('pointer-pixels');
    same(pixel.state.modes.mouseEncoding, 'SGR_PIXELS');
    await write('\x1b[?9l\x1b[?1006l'); const reset = await frame('pointer-reset-inactive');
    same(reset.state.modes.mouseProtocol, 'NONE'); same(reset.state.modes.mouseEncoding, 'DEFAULT');
    const outputs = reset.events.filter(event => ['data', 'binary'].includes(event.type)).length;
    await pointer(); const inactive = await frame('pointer-no-listeners');
    same(inactive.events.filter(event => ['data', 'binary'].includes(event.type)).length, outputs, 'Resetting inactive mutually exclusive mode removes active reports');
    await write('\x1b[?1004h'); await call('blur'); await call('focus'); const focus = await frame('focus-reports');
    check(focus.events.some(event => event.type === 'data' && event.value === '\x1b[I'), 'Actual focus-in report');
    check(focus.events.some(event => event.type === 'data' && event.value === '\x1b[O'), 'Actual focus-out report');
    await write('\x1b[?1004l'); await call('blur'); await call('focus'); await frame('focus-reset');
  } else throw new Error(`Missing actual mode scenario ${caseId}`);
  const last = frames.at(-1);
  return { assertions, capture: {
    cells: frames.map(frame => ({ label: frame.label, normal: clone(frame.state.normal.lines), alternate: clone(frame.state.alternate.lines) })),
    cursor: frames.map(frame => ({ label: frame.label, active: frame.state.active, normal: [frame.state.normal.cursorX, frame.state.normal.cursorY], alternate: [frame.state.alternate.cursorX, frame.state.alternate.cursorY] })),
    margins: frames.map(frame => ({ label: frame.label, normal: [frame.state.normal.scrollTop, frame.state.normal.scrollBottom], alternate: [frame.state.alternate.scrollTop, frame.state.alternate.scrollBottom] })),
    buffers: frames.map(frame => ({ label: frame.label, normal: clone(frame.state.normal), alternate: clone(frame.state.alternate) })),
    events: clone(last.events), replies: clone(last.events.filter(event => ['data', 'binary'].includes(event.type))), pixels,
    properties: { inputs: clone(inputs), steps: frames.map(frame => ({ label: frame.label, state: clone(frame.state), properties: clone(frame.properties) })) }
  } };
}
