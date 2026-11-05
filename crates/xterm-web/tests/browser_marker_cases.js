// Source-derived cases shared by the original Terminal oracle and Rust WASM.
const write = (term, text) => new Promise(resolve => term.write(text, resolve));
const require = (value, message) => { if (!value) throw new Error(message); };

export function markerValidationCase(term) {
  try {
    const values = [undefined, null, false, true, 2, -2, '2', '', ' 2 ', 9007199254740992];
    const lines = values.map(value => term.registerMarker(value).line);
    const errors = [NaN, Infinity, -Infinity, 0.5, 'bad', '1.5'].map(value => {
      try { term.registerMarker(value); } catch (error) { return error.message; }
      throw new Error('noninteger accepted');
    });
    const hints = [];
    const marker = term.registerMarker({ [Symbol.toPrimitive](hint) {
      hints.push(hint);
      return hint === 'default' ? '7' : 1;
    } });
    const token = {};
    for (const value of [{ valueOf() { throw token; } }, Symbol('offset'), 1n]) {
      try { term.registerMarker(value); throw new Error('offset should throw'); }
      catch (error) {
        if (typeof value === 'object') require(error === token, 'coercion throw identity');
        else require(error instanceof TypeError, 'symbol/bigint must throw TypeError');
      }
    }
    return { lines, errors, hints, objectLine: marker.line };
  } finally { term.dispose(); }
}

export async function markerLifecycleCase(term) {
  try {
    const list = term.markers;
    require(Array.isArray(list) && list === term.markers, 'marker list identity/Array shape');
    const first = term.registerMarker();
    const second = term.registerMarker(2);
    require(list.length === 2 && list[0] === first && list[1] === second, 'live marker list/handle identity');
    require(Object.keys(list).join(',') === '0,1' && list.map(value => value.line).join(',') === '0,2', 'list enumeration');
    require(first.onDispose === first.onDispose, 'event function identity');
    require(second.id === first.id + 1, 'global sequential IDs');
    const seen = [];
    const receiver = {};
    const collection = [];
    first.onDispose(function (value) {
      seen.push([this === receiver, value === undefined, first.isDisposed, first.line]);
    }, receiver, collection);
    const cancelled = first.onDispose(() => { throw new Error('disposed subscription fired'); });
    cancelled.dispose();
    require(collection.length === 1 && typeof collection[0].dispose === 'function', 'subscription collection');
    await write(term, 'a\r\nb\r\nc\r\nd\r\ne\r\n');
    require(first.isDisposed && first.line === -1, 'trim did not invalidate marker');
    first.dispose();
    first.dispose();
    second.dispose();
    require(list.length === 0, 'saved list membership after disposal');
    const third = term.registerMarker();
    const token = {};
    third.onDispose(() => { throw token; });
    try { third.dispose(); throw new Error('listener error hidden'); }
    catch (error) { require(error === token, 'disposal throw identity'); }
    third.dispose();
    return { seen, disposed: [first.isDisposed, second.isDisposed, third.isDisposed],
      lines: [first.line, second.line, third.line], remaining: term.markers.length };
  } finally { term.dispose(); }
}

export function markerResourcesCase(term) {
  try {
    const log = [];
    const marker = term.registerMarker();
    let gets = 0;
    const resource = { get dispose() {
      gets++;
      return function () {
        require(this === resource, 'resource receiver');
        require(marker.isDisposed && marker.line === -1, 'resource invalidation order');
        log.push('resource');
        marker.dispose();
        marker.register({ dispose() { log.push('appended'); } });
      };
    } };
    require(marker.register(resource) === resource, 'register return identity');
    marker.register(resource);
    require(gets === 0, 'resource inspected during registration');
    marker.onDispose(() => {
      log.push('event');
      marker.register({ dispose() { log.push('event-added'); } });
    });
    marker.dispose();
    marker.dispose();
    marker.register({ dispose() { throw new Error('late registration disposed'); } });
    marker.dispose();

    const changed = term.registerMarker();
    const mutable = { dispose() { throw new Error('captured stale method'); } };
    changed.register(mutable);
    mutable.dispose = function () { require(this === mutable, 'changed receiver'); log.push('changed'); };
    changed.dispose();

    const errors = [];
    for (const kind of ['method', 'getter', 'null', 'noncallable']) {
      const current = term.registerMarker();
      const token = {};
      const bad = kind === 'method' ? { dispose() { throw token; } }
        : kind === 'getter' ? { get dispose() { throw token; } }
        : kind === 'null' ? null : { dispose: 7 };
      require(current.register(bad) === bad, 'raw resource rejected during registration');
      current.register({ dispose() { throw new Error('continued after resource error'); } });
      let caught = false;
      try { current.dispose(); }
      catch (error) {
        caught = true;
        errors.push(kind === 'method' || kind === 'getter' ? error === token : error instanceof TypeError);
      }
      require(caught, 'resource error hidden');
      require(current.isDisposed && current.line === -1, 'failed resource marker state');
      current.dispose();
    }
    const eventFirst = term.registerMarker();
    const token = {};
    eventFirst.onDispose(() => { throw token; });
    eventFirst.register({ dispose() { throw new Error('resource ran after event failure'); } });
    try { eventFirst.dispose(); throw new Error('listener error hidden'); }
    catch (error) { require(error === token, 'event error changed by resource adapter'); }
    eventFirst.dispose();
    return { log, gets, errors };
  } finally { term.dispose(); }
}

export async function markerBufferCase(term) {
  try {
    await write(term, 'a\r\nb\r\nc');
    const normal = term.registerMarker(-1);
    const normalList = term.markers;
    await write(term, '\x1b[?1049h');
    require(term.markers.length === 0, 'alternate list contains normal marker');
    const alternate = term.registerMarker(1);
    const alternateList = term.markers;
    require(alternateList !== normalList && normalList[0] === normal, 'per-buffer list identity');
    await write(term, '\x1b[?1049l');
    require(alternateList.length === 0, 'saved alternate list after clearing');
    require(term.markers === normalList && term.markers.length === 1 && term.markers[0] === normal, 'normal marker identity after activation');
    const before = normal.line;
    const hints = [];
    const captured = term.registerMarker({ [Symbol.toPrimitive](hint) {
      hints.push(hint);
      if (hint === 'default') term.reset();
      return 1;
    } });
    return { before, alternateDisposed: alternate.isDisposed, capturedLine: captured.line,
      capturedInCurrentBuffer: term.markers.includes(captured), hints };
  } finally { term.dispose(); }
}
