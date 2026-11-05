// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
return (async () => {
  const methods = ['getImageAtBufferCell', 'extractTileAtBufferCell'];
  const image = '\x1b]1337;File=inline=1;width=2px;height=2px;preserveAspectRatio=0:cW9pZgAAAAIAAAACBAD/AQID//8BAgP//wECA///AQID/wAAAAAAAAAB\x07';
  const observations = [];
  const json = value => JSON.stringify(value);
  function check(condition, message) { if (!condition) throw Error(message); }
  function canvas(value) {
    if (value === undefined) return ['undefined'];
    return ['canvas', value.width, value.height,
      [...value.getContext('2d').getImageData(0, 0, 1, 1).data]];
  }
  function attempt(fn, sentinel) {
    try { return ['return', canvas(fn())]; }
    catch (error) { return ['throw', error === sentinel, error === undefined, error?.name]; }
  }
  async function fixture(api, populated) {
    const terminal = new api.Terminal({cols:20, rows:3, scrollback:2,
      fontSize:14, allowProposedApi:true, logLevel:'off'});
    const parent = document.body.appendChild(document.createElement('div'));
    parent.style.cssText = 'position:fixed;top:0;left:0';
    terminal.open(parent);
    const addon = new api.ImageAddon({sixelSupport:false, kittySupport:false,
      iipSupport:populated, showPlaceholder:false});
    const others = [];
    let added = 0;
    addon.onImageAdded(() => added++);
    addon.activate(terminal);
    try {
      if (populated) {
        await new Promise(resolve => terminal.write('\r\n'.repeat(6), resolve));
        await new Promise(resolve => terminal.write(image, resolve));
        check(added === 1 && addon.storageUsage === 0.000016, 'actual QOI image was not stored');
        const pixels = canvas(addon.getImageAtBufferCell(0, 4));
        check(pixels[0] === 'canvas' && json(pixels[3]) === '[1,2,3,255]', 'actual QOI image pixels differ');
      }
    } catch (error) { addon.dispose(); terminal.dispose(); parent.remove(); throw error; }
    return {terminal, addon, others,
      activateAnother() {
        const next = new api.Terminal({cols:20,rows:3,scrollback:2,allowProposedApi:true,logLevel:'off'});
        others.push(next); addon.activate(next);
      },
      dispose() { addon.dispose(); terminal.dispose(); for (const t of others) t.dispose(); parent.remove(); }
    };
  }
  for (let apiIndex = 0; apiIndex < apis.length; apiIndex++) {
    const api = apis[apiIndex], rows = [], trace = [], sentinel = {sentinel:true};
    if (scenario === 'empty') {
      const f = await fixture(api, false);
      try {
        const poison = {[Symbol.toPrimitive](hint) {trace.push(['x', hint]); throw sentinel;}};
        const missing = [undefined, NaN, Infinity, -Infinity, 0.5, -0.5, 'bad', '0.5', {}];
        for (const method of methods) {
          for (let i = 0; i < missing.length; i++) {
            const before = trace.length;
            const result = attempt(() => f.addon[method](poison, missing[i]), sentinel);
            rows.push([method, 'missing', i, result]);
            if (apiIndex === 0) check(json(result) === '["return",["undefined"]]' && trace.length === before,
              'source must skip x for an absent row');
          }
          for (const value of [Symbol('y'), 1n]) {
            const before = trace.length;
            const result = attempt(() => f.addon[method](poison, value), sentinel);
            rows.push([method, 'invalid', result]);
            if (apiIndex === 0) check(result[0] === 'throw' && result[3] === 'TypeError' && trace.length === before,
              'source y error must precede x');
          }
          for (const thrown of [sentinel, undefined]) {
            const y = {[Symbol.toPrimitive](hint) {trace.push(['yThrow', hint]); throw thrown;}};
            const result = attempt(() => f.addon[method](poison, y), sentinel);
            rows.push([method, 'yThrow', result]);
            if (apiIndex === 0) check(result[0] === 'throw' && (thrown === sentinel ? result[1] : result[2]),
              'source must preserve thrown identity');
          }
          const y = {[Symbol.toPrimitive](hint) {trace.push(['y', hint]); return 0;}};
          const before = trace.length;
          rows.push([method, 'order', attempt(() => f.addon[method](poison, y), sentinel)]);
          if (apiIndex === 0) check(json(trace.slice(before)) === '[["y","default"],["x","number"]]',
            'source must evaluate y addition before x multiplication');
        }
      } finally {f.dispose();}
    } else if (scenario === 'raw') {
      const f = await fixture(api, true);
      try {
        const values = [undefined, null, false, true, -6, -1, -0, 0, 0.5, 1, 2, 3, 4, 5,
          1000001, NaN, Infinity, -Infinity, '', '0', '00', '1', '4', '9', '1e0', 'bad',
          [], [4], new Number(4), new String('4'), Symbol('y'), 4n,
          {valueOf() {trace.push('valueOf'); return 4;}},
          {[Symbol.toPrimitive](hint) {trace.push(['string',hint]); return '4';}},
          {[Symbol.toPrimitive](hint) {trace.push(['number',hint]); return 4;}}];
        const xvalues = [undefined, null, false, true, -0, 0, -1, 0.5, NaN, Infinity,
          '', '0', '00', '-0', '0.0', '0e0', [], [0], new Number(0), new String('00'), Symbol('x'), 0n,
          {[Symbol.toPrimitive](hint) {trace.push(['xHint',hint]); return 0;}},
          {[Symbol.toPrimitive](hint) {trace.push(['xKey',hint]); return hint === 'number' ? 0 : Symbol('key');}},
          {[Symbol.toPrimitive](hint) {trace.push(['xThrow',hint]); if(hint === 'string') throw undefined; return 0;}}];
        for (const method of methods) {
          for (let i = 0; i < xvalues.length; i++) rows.push([method, 'x', i, attempt(() => f.addon[method](xvalues[i], 4), sentinel)]);
          let differences = 0, hits = 0;
          for (let i = 0; i < values.length; i++) {
            const result = attempt(() => f.addon[method](0, values[i]), sentinel);
            if (result[0] === 'return' && result[1][0] === 'canvas') hits++;
            rows.push([method, i, result]);
          }
          for (let y = 0; y < 10; y++) {
            const numeric = attempt(() => f.addon[method](0, y), sentinel);
            const string = attempt(() => f.addon[method](0, String(y)), sentinel);
            if (json(numeric) !== json(string)) differences++;
            rows.push([method, 'numeric/string', y, numeric, string]);
          }
          if (apiIndex === 0) check(hits > 0 && differences > 0, 'source rotated-ring fixture must distinguish addition from numeric coercion');
        }
      } finally {f.dispose();}
    } else if (scenario === 'reentry') {
      for (const method of methods) {
        for (const phase of ['yGrow', 'yReactivate', 'xReactivate', 'yThrowAfterGrow']) {
          const f = await fixture(api, true);
          try {
            const x = phase === 'xReactivate' ? {[Symbol.toPrimitive](hint) {
              trace.push([phase, 'x', hint]); f.activateAnother(); return 0;
            }} : 0;
            const y = phase === 'xReactivate' ? 4 : {[Symbol.toPrimitive](hint) {
              trace.push([phase, 'y', hint]);
              if (phase === 'yReactivate') {f.activateAnother(); return 4;}
              f.terminal.resize(20, 4);
              if (phase === 'yThrowAfterGrow') throw undefined;
              return -1;
            }};
            const result = attempt(() => f.addon[method](x, y), sentinel);
            rows.push([method, phase, result, f.addon.storageUsage]);
            if (apiIndex === 0) {
              if (phase === 'yThrowAfterGrow') check(result[0] === 'throw' && result[2], 'source throw undefined must survive resizing');
              else check(result[0] === 'return' && result[1][0] === 'canvas' && json(result[1][3]) === '[1,2,3,255]',
                'source must retain original array/storage/line through ' + phase);
            }
          } finally {f.dispose();}
        }
      }
    } else throw Error('Unknown row scenario ' + scenario);
    observations.push({rows, trace});
  }
  check(json(observations[1]) === json(observations[0]), 'direct WASM row phases differ: ' + json(observations));
  check(json(observations[2]) === json(observations[0]), 'shipped bootstrap row phases differ: ' + json(observations));
})();
