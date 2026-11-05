// Run the same observation program against unchanged source and an opened Rust terminal.
return (run, makeEvent) => {
  const trace = [];
  const assert = (ok, message) => { if (!ok) throw new Error(message); };
  const modes = [
    'empty', 'ordinary', 'root', 'hover', 'truthy root', 'truthy hover',
    'second classList', 'contains getter', 'null target', 'missing classList',
    'null classList', 'missing contains', 'throw classList', 'throw contains getter',
    'throw contains call', 'throw path getter', 'throw path call', 'null path',
    'array-like', 'length coercion', 'live append', 'live truncate',
    'index getter', 'throw index', 'throw length', 'primitive path'
  ];
  for (const mode of modes) {
    const log = [], marker = { mode };
    let path, lists = 0;
    const target = label => {
      const list = { contains(name) {
        assert(this === list, `${mode}: contains receiver`);
        log.push(`${label}.contains:${name}`);
        if (mode === 'throw contains call') throw marker;
        if (mode === 'live append' && name === 'xterm') path.push(target('hover'));
        if (mode === 'live truncate' && name === 'xterm') path.length = 0;
        if (label === 'hover') return name === 'xterm-hover';
        if (mode === 'root') return name === 'xterm';
        if (mode === 'truthy root' && name === 'xterm') return '0';
        if (mode === 'truthy hover' && name === 'xterm-hover') return {};
        return false;
      } };
      if (mode === 'contains getter') {
        const contains = list.contains;
        Object.defineProperty(list, 'contains', { get() { log.push('get contains'); return contains; } });
      }
      if (mode === 'throw contains getter') Object.defineProperty(list, 'contains', { get() { throw marker; } });
      if (mode === 'missing contains') delete list.contains;
      const entry = {};
      Object.defineProperty(entry, 'classList', { get() {
        log.push(`${label}.classList`);
        if (mode === 'throw classList') throw marker;
        if (mode === 'null classList') return null;
        if (mode === 'second classList' && ++lists === 2) return { contains(name) {
          log.push(`second.contains:${name}`); return true;
        } };
        return list;
      } });
      return entry;
    };
    if (mode === 'empty') path = [];
    else if (mode === 'null target') path = [null];
    else if (mode === 'missing classList') path = [{}];
    else if (mode === 'null path') path = null;
    else if (mode === 'primitive path') path = 'x';
    else if (mode === 'array-like' || mode === 'length coercion') {
      path = { 0: target('hover'), get length() {
        log.push('get length');
        return mode === 'length coercion' ? { valueOf() { log.push('length.valueOf'); return 1; } } : 1;
      } };
    } else path = [target(mode === 'hover' ? 'hover' : 'target')];
    if (mode === 'index getter' || mode === 'throw index') {
      Object.defineProperty(path, '0', { get() {
        log.push('get index');
        if (mode === 'throw index') throw marker;
        return target('hover');
      } });
    }
    if (mode === 'throw length') path = { get length() { log.push('get length'); throw marker; } };
    const event = makeEvent();
    Object.defineProperty(event, 'composedPath', { get() {
      log.push('get composedPath');
      if (mode === 'throw path getter') throw marker;
      return function () {
        assert(this === event && arguments.length === 0, `${mode}: composedPath receiver/arguments`);
        log.push('call composedPath');
        if (mode === 'throw path call') throw marker;
        return path;
      };
    } });
    const observed = run(event, marker);
    const error = mode.startsWith('throw ') ? 'marker' : ['null target', 'missing classList', 'null classList',
      'missing contains', 'null path', 'primitive path'].includes(mode) ? 'TypeError' : 'none';
    const blocked = ['hover', 'truthy root', 'truthy hover', 'second classList', 'array-like',
      'length coercion', 'live append', 'index getter'].includes(mode);
    // Truthy root stops scanning but does not suppress hover handling.
    const provided = error === 'none' && (!blocked || mode === 'truthy root');
    assert(observed.error === error, `${mode}: expected ${error}, got ${observed.error}`);
    assert(observed.provided === provided, `${mode}: provided=${observed.provided}, expected ${provided}`);
    trace.push({ mode, log, ...observed });
  }
  return trace;
};
