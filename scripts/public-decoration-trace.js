return (async () => {
  const trace = [];
  const check = (condition, message) => { if (!condition) throw new Error(message); };
  const write = text => new Promise(resolve => terminal.write(text, resolve));
  const until = async predicate => {
    const end = performance.now() + 5000;
    while (!predicate()) {
      if (performance.now() >= end) throw new Error('Decoration render deadline');
      await new Promise(resolve => requestAnimationFrame(resolve));
    }
  };
  const parent = document.createElement('div');
  document.body.appendChild(parent);
  try {
    check(typeof terminal.registerDecoration === 'function', 'missing public registerDecoration');
    check(terminal.registerDecoration === other.registerDecoration, 'shared registration method identity');
    trace.push({ name: 'public method without proposed API', ok: true });
    const marker = terminal.registerMarker();
    const values = [undefined, null, false, 0, -0, NaN, '', '0'];
    for (const x of values) {
      const decoration = terminal.registerDecoration({ marker, x });
      check(decoration && !decoration.isDisposed, 'falsey or numeric-string x rejected');
      decoration.dispose();
    }
    const bad = [-1, 0.5, Infinity, -Infinity, 'bad'];
    for (const x of bad) {
      let message;
      try { terminal.registerDecoration({ marker, x }); } catch (error) { message = error.message; }
      check(message === 'This API only accepts positive integers', 'validation message');
    }
    trace.push({ name: 'source validation', accepted: values.length, rejected: bad.length });
    const order = [], sentinel = {};
    const optionsWithReads = { marker,
      get x() { order.push('x'); return { [Symbol.toPrimitive](hint) { order.push(hint); return 1; } }; },
      get width() { order.push('width'); return 1; },
      get height() { order.push('height'); return 1; }
    };
    const phases = terminal.registerDecoration(optionsWithReads);
    check(order.slice(0, 3).join() === 'x,width,height', 'validation did not evaluate all arguments first');
    phases.dispose();
    trace.push({ name: 'validation argument ordering', first: order.slice(0, 6) });
    const dead = terminal.registerMarker(); dead.dispose();
    check(terminal.registerDecoration({ marker: dead }) === undefined, 'disposed marker');
    let actual;
    try { terminal.registerDecoration({ marker: dead, get x() { throw sentinel; } }); } catch (error) { actual = error; }
    check(actual === sentinel, 'validation error identity before disposed guard');
    trace.push({ name: 'disposed guard and getter error identity', ok: true });
    let heightReads = 0;
    const partialMarker = other.registerMarker();
    actual = undefined;
    try {
      other.registerDecoration({ marker: partialMarker, get height() {
        if (++heightReads === 2) throw sentinel;
        return 1;
      } });
    } catch (error) { actual = error; }
    check(actual === sentinel && heightReads === 2, 'cache getter error identity and phase');
    other.reset();
    check(partialMarker.isDisposed, 'reset after partial registration');
    trace.push({ name: 'throwing registration and reset', ok: true });
    const foreign = other.registerMarker();
    const foreignDecoration = terminal.registerDecoration({ marker: foreign });
    check(foreignDecoration.marker === foreign, 'cross-terminal marker identity');
    foreign.dispose();
    check(foreignDecoration.isDisposed, 'cross-terminal marker disposal');
    trace.push({ name: 'cross-terminal native marker', ok: true });
    const overview = { color: 'red' };
    const options = { marker, x: 1, width: 2, height: 2, layer: 'top', backgroundColor: '#112233', overviewRulerOptions: overview };
    const decoration = terminal.registerDecoration(options);
    check(decoration.options === options && decoration.marker === marker, 'options or marker identity');
    check(overview.position === 'full', 'live overview default');
    check(decoration.element === undefined, 'preopen element');
    check(decoration.onRender === decoration.onRender && decoration.onDispose === decoration.onDispose, 'event function identity');
    let renders = 0;
    const context = {}, tokens = [];
    decoration.onRender(function (element) {
      check(this === context && element === decoration.element, 'onRender receiver or element identity');
      renders++;
    }, context, tokens);
    check(tokens.length === 1 && typeof tokens[0].dispose === 'function', 'event token collection');
    terminal.options.scrollbar = { width: 15, overviewRuler: {} };
    terminal.open(parent);
    await write('abc');
    await until(() => decoration.element && renders > 0);
    check(decoration.element.isConnected && decoration.element.classList.contains('xterm-decoration-top-layer'), 'live overlay');
    let cell = terminal.dimensions.css.cell;
    check(decoration.element.style.width === `${Math.round(2 * cell.width)}px`, 'initial width');
    check(decoration.element.style.height === `${2 * cell.height}px`, 'initial height');
    trace.push({ name: 'preopen registration and live render event', ok: true });
    const hasBackground = () => Array.from(parent.querySelectorAll('.xterm-rows span'))
      .some(element => getComputedStyle(element).backgroundColor === 'rgb(17, 34, 51)');
    await until(hasBackground);
    options.backgroundColor = '#445566';
    const colorsBefore = renders;
    terminal.refresh(0, 3);
    await until(() => renders > colorsBefore);
    check(hasBackground(), 'parsed decoration color must remain cached after option mutation');
    trace.push({ name: 'live row decoration and cached colors', ok: true });
    const canvas = parent.querySelector('.xterm-decoration-overview-ruler');
    check(canvas, 'overview ruler canvas');
    const pixel = () => Array.from(canvas.getContext('2d').getImageData(2, 0, 1, 1).data);
    await until(() => pixel().join() === '255,0,0,255');
    overview.color = 'blue';
    terminal.refresh(0, 3);
    await until(() => pixel().join() === '0,0,255,255');
    trace.push({ name: 'live overview color on actual canvas', ok: true });
    options.width = 3; options.height = 1; options.x = 2; options.anchor = 'right';
    const before = renders;
    terminal.resize(21, 4); terminal.refresh(0, 3);
    await until(() => renders > before && decoration.element.style.width === `${Math.round(3 * terminal.dimensions.css.cell.width)}px`);
    cell = terminal.dimensions.css.cell;
    check(decoration.element.style.height === `${cell.height}px`, 'mutated height');
    // CSSOM serializes fractional positions to browser precision. Normalize
    // the expected value through the same platform parser, not a tolerance.
    const expectedPosition = document.createElement('div').style;
    expectedPosition.right = `${2 * cell.width}px`;
    check(decoration.element.style.right === expectedPosition.right, 'mutated anchor and x');
    check(decoration.options === options, 'options replaced with snapshot');
    trace.push({ name: 'live mutable geometry', ok: true });
    tokens[0].dispose();
    let failures = 0;
    const throwing = terminal.registerDecoration({ marker });
    throwing.onDispose(() => { failures++; if (failures === 1) throw sentinel; });
    actual = undefined;
    try { throwing.dispose(); } catch (error) { actual = error; }
    check(actual === sentinel && !throwing.isDisposed, 'dispose listener throw identity or phase');
    throwing.dispose();
    check(throwing.isDisposed && failures === 2, 'failed dispose not retried correctly');
    trace.push({ name: 'fallible disposal and retry', ok: true });
    let calls = 0;
    decoration.onDispose(() => calls++);
    decoration.dispose();
    check(decoration.isDisposed && !decoration.element.isConnected, 'dispose overlay removal');
    const after = calls; decoration.dispose();
    check(calls === after, 'repeat disposal callback replay');
    trace.push({ name: 'overlay disposal', calls: after });
    const reset = terminal.registerDecoration({ marker: terminal.registerMarker() });
    terminal.reset();
    check(reset.isDisposed, 'terminal reset decoration lifecycle');
    trace.push({ name: 'terminal reset', ok: true });
    const pending = terminal.registerDecoration({ marker: terminal.registerMarker() });
    terminal.dispose();
    check(pending.isDisposed && parent.childElementCount === 0, 'terminal dispose decoration lifecycle');
    trace.push({ name: 'terminal disposal', ok: true });
    return trace;
  } finally {
    terminal.dispose(); other.dispose(); parent.remove();
  }
})();
