// Licensed under MIT. Deterministic loader for unchanged upstream input suites.
import assert from 'node:assert/strict';
import { stripTypeScriptTypes } from 'node:module';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

export const hex = value => value.length ? Array.from({ length: value.length }, (_, i) => value.charCodeAt(i).toString(16)).join(',') : '~';
export const row = (action, text = '', number = '-', flag = '-') => [action, hex(text), number, flag].join('\t');
const sourcePaths = ['source_repo/src/browser/Clipboard.ts', 'source_repo/src/browser/input/CompositionHelper.ts'];
export const referencePaths = sourcePaths.flatMap(name => [name, name.replace('.ts', '.test.ts')]);

export function loadSource(root) {
  const read = name => readFileSync(new URL(name, root), 'utf8');
  const erase = text => stripTypeScriptTypes(text.replace(/^import .*;\r?$/gm, '').replace(/^export /gm, '')
    .replace(/@(IBufferService|IOptionsService|ICoreService|IRenderService)\s+/g, ''), { mode: 'transform' });
  const tests = [], groups = [], queue = [], traces = [], clipboard = [];
  let timerId = 0, current, activeName = '', assertions = 0;
  const snapshot = (continuing = '-') => [Number(current.helper.isComposing), continuing,
    hex(current.view.textContent), current.data.length ? current.data.map(hex).join('/') : '-',
    queue.filter(item => item.modeled).length].join(';');
  const record = (action, text = '', number = '-', continuing = '-') => {
    traces.push({ input: row(action, text, number), expected: snapshot(continuing), name: activeName });
  };
  function schedule(fn, delay) {
    assert.equal(delay, 0, 'oracle supports the source zero-delay task contract only');
    const text = fn.toString();
    const modeled = text.includes('_compositionPosition.end =') || text.includes('_isSendingComposition)') || text.includes('_textareaChangeTimer = undefined');
    const geometry = text.includes('this.updateCompositionElements(true)');
    const item = { id: ++timerId, fn, modeled, geometry };
    queue.push(item);
    return item.id;
  }
  function tick(modeledOnly = false) {
    let steps = 0;
    while (queue.length) {
      assert.ok(++steps < 10000, 'unbounded oracle scheduling');
      const task = queue.shift();
      task.fn();
      if (task.modeled) record('tick');
      if (!modeledOnly || task.modeled) return true;
      assert.ok(task.geometry, 'unexpected driver callback in synthetic trace');
    }
    return false;
  }
  const context = vm.createContext({
    assert: {
      equal: (actual, expected, message) => { assertions++; assert.equal(actual, expected, message); },
      strictEqual: (actual, expected, message) => { assertions++; assert.strictEqual(actual, expected, message); }
    },
    describe: (name, fn) => { groups.push({ name, hooks: [] }); fn(); groups.pop(); },
    beforeEach: fn => groups.at(-1).hooks.push(fn),
    it: (name, fn) => tests.push({ name: [...groups.map(group => group.name), name].join(' / '), hooks: groups.flatMap(group => group.hooks), fn }),
    setTimeout: schedule,
    window: { setTimeout: schedule },
    MockCoreService: class { triggerDataEvent() {} },
    MockBufferService: class { constructor(cols, rows) { this.cols = cols; this.rows = rows; this.buffer = { x: 0, y: 0, isCursorInViewport: true }; } },
    MockOptionsService: class { rawOptions = { fontFamily: 'monospace', fontSize: 15 }; },
    MockRenderService: class { dimensions = { css: { cell: { width: 0, height: 0 } } }; }
  });
  vm.runInContext(erase(read('source_repo/src/common/data/EscapeSequences.ts')), context);
  vm.runInContext(erase(read(sourcePaths[0])) + '\nthis.Clipboard = { prepareTextForTerminal, bracketTextForPaste, paste };', context);
  vm.runInContext(erase(read(sourcePaths[1])) + '\nthis.SourceComposition = CompositionHelper;', context);
  const SourceComposition = context.SourceComposition;
  class TracedComposition extends SourceComposition {
    constructor(textarea, view, buffer, options, core, render) {
      super(textarea, view, buffer, options, core, render);
      current = { helper: this, textarea, view, data: [] };
      const send = core.triggerDataEvent.bind(core);
      core.triggerDataEvent = (...args) => { current.data.push(args[0]); send(...args); };
      for (const [property, action] of [['value', 'value'], ['selectionStart', 'startsel'], ['selectionEnd', 'endsel']]) {
        let value = textarea[property];
        Object.defineProperty(textarea, property, { get: () => value, set: next => {
          value = next;
          record(action, property === 'value' ? next : '', property === 'value' ? '-' : next ?? '-');
        } });
      }
      record('reset');
    }
    compositionstart() { super.compositionstart(); record('start'); }
    compositionupdate(event) { super.compositionupdate(event); record('update', event.data); }
    compositionend() { super.compositionend(); record('end'); }
    keydown(event) { const result = super.keydown(event); record('key', '', event.keyCode, Number(result)); return result; }
  }
  context.TracedComposition = TracedComposition;
  vm.runInContext('CompositionHelper = TracedComposition;', context);
  const plainClipboard = { ...context.Clipboard };
  for (const [name, action] of [['prepareTextForTerminal', 'prepare'], ['bracketTextForPaste', 'bracket']]) {
    const original = plainClipboard[name];
    context.Clipboard[name] = (...args) => {
      const output = original(...args);
      clipboard.push({ input: row(action, args[0], Number(args[1] || false)), expected: hex(output), name: activeName });
      return output;
    };
  }
  function runOriginal() {
    for (const name of sourcePaths.map(path => path.replace('.ts', '.test.ts'))) vm.runInContext(erase(read(name)), context);
    for (const test of tests) {
      queue.length = 0;
      activeName = test.name;
      test.hooks.forEach(fn => fn());
      let done = false;
      test.fn(() => { done = true; });
      let limit = 0;
      while (queue.length) { assert.ok(++limit < 10000, 'upstream timer loop'); tick(); }
      if (test.fn.length) assert.ok(done, `${test.name} did not complete`);
    }
    return { tests: tests.map(test => test.name), assertions, traces: [...traces], clipboard: [...clipboard] };
  }
  function reset(name) {
    queue.length = 0;
    activeName = name;
    const textarea = { value: '', style: {} };
    const view = { textContent: '', style: {}, classList: { add() {}, remove() {} }, getBoundingClientRect: () => ({ width: 0 }) };
    return new TracedComposition(textarea, view, new context.MockBufferService(10, 5), new context.MockOptionsService(), new context.MockCoreService(), new context.MockRenderService());
  }
  function apply(action, text = '', number = '-') {
    switch (action) {
      case 'reset': reset(text); break;
      case 'value': current.textarea.value = text; break;
      case 'startsel': current.textarea.selectionStart = number === '-' ? undefined : number; break;
      case 'endsel': current.textarea.selectionEnd = number === '-' ? undefined : number; break;
      case 'start': current.helper.compositionstart(); break;
      case 'update': current.helper.compositionupdate({ data: text }); break;
      case 'end': current.helper.compositionend(); break;
      case 'key': current.helper.keydown({ keyCode: number }); break;
      case 'tick': tick(true); break;
      default: throw new Error(`unknown source operation ${action}`);
    }
  }
  return { runOriginal, apply, traces, clipboard: plainClipboard, pending: () => queue.some(task => task.modeled) };
}
