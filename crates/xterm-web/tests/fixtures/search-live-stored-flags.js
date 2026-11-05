// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
async function checkSearchCallArgs(term, addon, write, equal) {
  await write('foo FOO foo');
  const decorations = {matchOverviewRuler:'#ff0000',activeMatchColorOverviewRuler:'#00ff00'};
  const keys = ['caseSensitive','regex','wholeWord'];
  let cases = 0;
  async function check(method, nested, configure, verify) {
    const child = new addon.constructor(); term.loadAddon(child);
    const trace = [], events = [];
    let armed = false, entered = false;
    const nestedFind = options => {
      const previous = armed; armed = false;
      try { return child[nested]('foo', options); }
      finally { armed = previous; }
    };
    try {
      equal(child.findNext('foo'), true);
      child.onBeforeSearch(() => events.push('before'));
      child.onAfterSearch(() => events.push('after'));
      const state = configure({trace, nestedFind, armed:() => armed});
      const request = {get decorations() {
        if (!entered) { entered = true; equal(nestedFind(state.stored), true); armed = true; }
        return decorations;
      }};
      keys.forEach((key, index) => Object.defineProperty(request, key, {get() {
        if (!armed) return undefined;
        trace.push('incoming.' + key);
        return state.incoming(index);
      }}));
      let caught, threw = false;
      try { child[method]('foo', request); }
      catch (error) { caught = error; threw = true; }
      equal(threw, true);
      verify({trace, events, caught, selected:term.hasSelection()});
      cases++;
    } finally { child.dispose(); }
  }
  const prefix = index => keys.slice(0,index+1).flatMap(key => ['stored.'+key,'incoming.'+key]);
  for (const method of ['findNext','findPrevious']) {
    for (const nested of ['findNext','findPrevious']) {
      // Reentry can replace lastSearchOptions in either operand. The request
      // for the next property is live; the current right operand is incoming.
      for (const side of ['stored','incoming']) {
        for (const index of [0,1]) {
          for (const replacement of [undefined,null,false,0,NaN,0n,'',{}]) {
            const sentinel = {}, object = replacement && typeof replacement === 'object';
            await check(method,nested,({trace,nestedFind,armed}) => {
              const replace = () => {
                try { equal(nestedFind(replacement), true); }
                catch (error) { if (replacement !== null || !(error instanceof TypeError)) throw error; }
              };
              if (object) keys.forEach(key => Object.defineProperty(replacement,key,{get() {
                if (!armed()) return undefined;
                trace.push('replacement.'+key); throw sentinel;
              }}));
              const stored = {};
              keys.forEach((key,at) => Object.defineProperty(stored,key,{get() {
                if (!armed()) return undefined;
                trace.push('stored.'+key);
                if (side === 'stored' && at === index) replace();
                return undefined;
              }}));
              return {stored, incoming(at) {
                if (at === index && side === 'incoming') replace();
                if (at > index) throw sentinel;
                return undefined;
              }};
            },({trace,events,caught,selected}) => {
              const expected = prefix(index);
              if (replacement != null) expected.push((object ? 'replacement.' : 'incoming.')+keys[index+1]);
              equal(trace, expected);
              equal(replacement == null ? caught instanceof TypeError : caught === sentinel, true);
              equal(selected, replacement !== null);
              equal(events, replacement === null ? ['before','before','after','before'] : ['before','before','after','before','after']);
            });
          }
        }
      }
      // Strict equality must not coerce flags to matching booleans. Mismatch
      // highlights before matching; equality clears selection before matching.
      const shared = {}, symbol = Symbol('shared');
      const pairs = [[undefined,undefined,false],[false,undefined,true],[true,1,true],
        [0,false,true],['',false,true],[null,undefined,true],[NaN,NaN,true],[0,-0,false],
        [shared,shared,false],[{},{},true],[1n,1,true],[1n,1n,false],[symbol,symbol,false],
        [Symbol('x'),Symbol('x'),true],['x','x',false],['x',Object('x'),true]];
      for (const index of [0,1,2]) {
        for (const [left,right,changed] of pairs) {
          const sentinel = {}; let matching = false;
          await check(method,nested,({trace,armed}) => {
            const stored = {};
            keys.forEach((key,at) => Object.defineProperty(stored,key,{get() {
              if (!armed()) return undefined;
              trace.push('stored.'+key); return at === index ? left : undefined;
            }}));
            return {stored, incoming(at) {
              if (matching) { trace[trace.length-1] = 'matching.'+keys[at]; throw sentinel; }
              if ((at === index && changed) || at === 2) matching = true;
              return at === index ? right : undefined;
            }};
          },({trace,events,caught,selected}) => {
            equal(caught === sentinel, true);
            equal(trace,[...prefix(changed ? index : 2),'matching.regex']);
            equal(selected,changed);
            equal(events,['before','before','after']);
          });
        }
      }
      for (const side of ['stored','incoming']) {
        for (const index of [0,1,2]) {
          for (const sentinel of [undefined,null,0,'failure',Symbol('failure'),{}]) {
            await check(method,nested,({trace,armed}) => {
              const stored = {};
              keys.forEach((key,at) => Object.defineProperty(stored,key,{get() {
                if (!armed()) return undefined;
                trace.push('stored.'+key);
                if (side === 'stored' && at === index) throw sentinel;
                return undefined;
              }}));
              return {stored, incoming(at) {
                if (side === 'incoming' && at === index) throw sentinel;
                return undefined;
              }};
            },({trace,events,caught,selected}) => {
              const expected = prefix(index);
              if (side === 'stored') expected.pop();
              equal(trace,expected); equal(Object.is(caught,sentinel),true);
              equal(selected,true); equal(events,['before','before','after']);
            });
          }
        }
      }
    }
  }
  equal(cases,464);
  return cases;
}
