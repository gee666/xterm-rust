// Test-only raw timer values shared by intact source and actual Rust RenderService.
import { withRenderHarness as withHandleHarness } from './render_service_host.mjs';
const handleKinds = ['undefined', 'null', 'false', 'true', 'zero', 'negative-zero',
  'fraction', 'nan', 'infinity', 'wide', 'string', 'empty-string', 'symbol', 'bigint', 'object', 'function'];
const handleActions = ['flush', 'dispose', 'deliver'];
const handlePhases = ['get-buffer', 'call-buffer', 'get-flush', 'call-flush',
  'get-dispose', 'call-dispose', 'call-deliver', 'clear-get-replace', 'clear-call-replace',
  ...['get', 'call'].flatMap(phase => ['object', 'undefined', 'null', 'string'].map(kind => `${phase}-throw-${kind}`))];
export const syncHandlePrograms = [
  ...handleKinds.flatMap(kind => handleActions.map(action => `${kind}/${action}`)),
  ...['object', 'symbol', 'null', 'undefined'].flatMap(kind => handlePhases.map(action => `${kind}/${action}`))
];
export function runSyncHandleProgram(make, program) {
  return withHandleHarness(host => {
    const dispatch = make(host);
    const log = [];
    const [kind, action] = program.split('/');
    const records = [];
    const command = (op, extra = {}) => {
      try { return dispatch({ op, ...extra }); }
      finally { log.push(...dispatch({ op: 'drain' })); }
    };
    const mode = value => command('mode', { value });
    const refresh = (start = 1, end = 2) => command('refresh', { start, end });
    const failureKind = action.split('-').at(-1);
    const failure = ({ object: Object.freeze({ sentinel: program }), undefined: undefined,
      null: null, string: 'sync-set-failure' })[failureKind];
    const originalSet = host.window.setTimeout;
    const originalClear = host.window.clearTimeout;
    let fired = false;
    function value(index) {
      const converted = () => { log.push(`unexpected-conversion:${index}`); throw new Error('Timer handle converted'); };
      switch (kind) {
        case 'undefined': return undefined;
        case 'null': return null;
        case 'false': return false;
        case 'true': return true;
        case 'zero': return 0;
        case 'negative-zero': return -0;
        case 'fraction': return 1.25;
        case 'nan': return NaN;
        case 'infinity': return Infinity;
        case 'wide': return 2 ** 40 + 17;
        case 'string': return 'raw-timer';
        case 'empty-string': return '';
        case 'symbol': return Symbol(`timer-${index}`);
        case 'bigint': return 2n ** 80n + BigInt(index);
        case 'object': return { [Symbol.toPrimitive]: converted, valueOf: converted, toString: converted };
        case 'function': return Object.assign(function() {}, { [Symbol.toPrimitive]: converted });
        default: throw new Error(`Unknown handle kind ${kind}`);
      }
    }
    function deliver(index) {
      const record = records[index];
      if (!record) throw new Error('Missing retained timer');
      log.push(`deliver:${index}:${record.cleared}`);
      record.fn();
      host.frame();
    }
    function hook(phase) {
      if (fired || !action.startsWith(phase + '-')) return;
      fired = true;
      log.push(`hook:${phase}`);
      if (action.includes('buffer')) { mode(true); refresh(0, 4); }
      if (action.includes('flush') || action.includes('replace')) { mode(false); refresh(3, 3); }
      if (action.includes('replace')) { mode(true); refresh(2, 4); }
      if (action.includes('dispose')) command('dispose');
      if (action.includes('deliver')) deliver(records.length - 1);
      if (action.includes('throw')) throw failure;
    }
    Object.defineProperty(host.window, 'setTimeout', { configurable: true, get() {
      log.push('get-set');
      hook('get');
      const schedule = function(fn, delay) {
        const index = records.length;
        const handle = value(index);
        records.push({ fn, handle, cleared: false });
        log.push(`set:${index}:${delay}:${this === host.window}`);
        hook('call');
        return handle;
      };
      // Invocation must not use an overridable Function.prototype.call property.
      schedule.call = () => { throw new Error('setTimeout own .call invoked'); };
      return schedule;
    } });
    Object.defineProperty(host.window, 'clearTimeout', { configurable: true, get() {
      log.push('get-clear');
      hook('clear-get');
      return function(handle) {
        const index = records.findLastIndex(record => Object.is(record.handle, handle));
        log.push(`clear:${index}:${handle === undefined}:${Object.is(handle, -0)}:${this === host.window}`);
        if (index < 0 && handle !== undefined) throw new Error('Coerced or foreign timer handle');
        hook('clear-call');
        for (const record of records) if (Object.is(record.handle, handle)) record.cleared = true;
      };
    } });
    function attempt(label, callback) {
      try { callback(); log.push(`${label}:ok`); }
      catch (error) {
        if (!action.includes('throw') || error !== failure) throw error;
        log.push(`${label}:same-error`);
      }
      log.push(`state:${command('state')}:registrations:${records.length}:${JSON.stringify(host.pending())}`);
    }
    try {
      mode(true);
      attempt('buffer', () => refresh());
      attempt('buffer-again', () => refresh(0, 4));
      if (action === 'deliver' || action.includes('throw') && records.length) {
        attempt('retained', () => deliver(0));
      }
      mode(false);
      if (action === 'dispose') attempt('dispose-initial', () => command('dispose'));
      else attempt('flush', () => refresh(1, 1));
      // Cancellation must not invalidate any callback a host retained.
      for (let index = 0; index < records.length; index++) attempt(`retained-${index}`, () => deliver(index));
      mode(true); attempt('new-buffer', () => refresh(2, 2));
      mode(false); attempt('new-flush', () => refresh(0, 0));
      attempt('dispose', () => command('dispose'));
      return log;
    } finally {
      Object.defineProperty(host.window, 'setTimeout', { configurable: true, writable: true, value: originalSet });
      Object.defineProperty(host.window, 'clearTimeout', { configurable: true, writable: true, value: originalClear });
      command('dispose');
    }
  });
}
export function runSyncHandleCases(make) {
  return syncHandlePrograms.map(program => ({ program, log: runSyncHandleProgram(make, program) }));
}
