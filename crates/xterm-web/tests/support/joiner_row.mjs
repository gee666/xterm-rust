// Identical row inputs for unchanged TypeScript and the Rust/WASM planner.
export function joinerRowScenarios(create) {
  const results = {};
  function run(name, make, options = {}) {
    const probe = create();
    const trace = [], marker = {};
    try {
      const ranges = make(trace, marker, probe);
      let output;
      try { output = {html: probe.render(ranges, options)}; }
      catch (error) { output = {error: error === marker ? 'marker' : error.name}; }
      results[name] = {...output, trace};
    } finally { probe.free?.(); }
  }
  const values = [0, 1, 2, 2.5, -1, 9, '3', null, undefined, NaN, -0, 3n];
  for (let i = 0; i < values.length; ++i) run(`end${i}`, () => [[1, values[i]]]);
  for (const [name, start] of [['string','1'],['negative',-1],['fraction',0.5],['false',false],['object',{}]]) {
    run(`strict${name}`, () => [{0:start, get 1() { throw new Error('unreachable endpoint'); }}]);
  }
  run('empty', () => []);
  run('skipWide', () => [[2,4],[3,5]]);
  run('cursorReject', () => [[1,4],[4,6]], {cursor:true, x:3});
  run('selectionReject', () => [[1,4],[4,6]], {selection:[2,4]});
  run('columnReject', () => [[1,4],[4,6]], {selection:[2,4], column:true});
  run('objectEnd', trace => [[1,{[Symbol.toPrimitive](hint) {trace.push(hint); return 4;}}]]);
  run('symbolEnd', () => [[1,Symbol('end')]]);
  run('nullHead', () => [null]);
  run('falseHead', () => [false,[1,4]]);
  run('lateStartThrow', (_,marker) => [{get 0() { throw marker; },1:4}]);
  run('headOnly', trace => [{get 0() { trace.push('start'); return '1'; },get 1() {trace.push('end'); return 4;}}]);
  run('readOrder', trace => new Proxy([new Proxy([1,4],{
    get(t,k) {trace.push('range:'+String(k)); return Reflect.get(t,k);}
  })], {get(t,k) {trace.push('outer:'+String(k)); return Reflect.get(t,k);}}));
  run('lateEndThrow', (trace,marker) => {
    let reads=0;
    return [{0:1,get 1() {trace.push(++reads); if(reads===4) throw marker; return 4;}}];
  });
  run('shiftIdentity', (trace,marker) => {
    const range={get 0() {trace.push('shifted:start'); return 1;},get 1() {trace.push('shifted:end');return 4;}};
    return {0:[1,3],length:1,shift() {trace.push(this.length);this.length=0;return range;}};
  });
  run('shiftError', (_,marker) => ({0:[1,3],length:1,shift() {throw marker;}}));
  run('lengthError', (_,marker) => ({get length() {throw marker;}}));
  run('cellMutation', (trace,_,probe) => [{0:1,get 1() {probe.set(3,90,1); trace.push('mutate'); return 4;}}]);
  run('selectionMutation', (trace,_,probe) => [{0:1,get 1() {probe.selection(2,4);trace.push('select');return 4;}}]);
  run('startAfterMatch', trace => {
    let reads=0;
    return [{get 0() {trace.push(++reads); return reads===1 ? 1 : '1';},1:4}];
  });
  run('endpointChanging', trace => {
    let reads=0;
    return [{0:1,get 1() {const n=++reads;trace.push(n);return n < 4 ? 4 : 5;}}];
  });
  run('recursiveWorkCell', (trace,_,probe) => {
    let nested=false;
    return [{get 0() {
      if(!nested) {nested=true;trace.push(probe.render([],{}));}
      return '1';
    },1:4}];
  });
  run('cursorMutationRevisit', () => {
    let reads=0;
    return [{0:1,get 1() {return ++reads===4 ? 2 : 4;}},[4,1]];
  }, {cursor:true,x:3});
  let seed=0x348C0DE;
  for(let i=0;i<256;i++) {
    seed=(Math.imul(seed,1664525)+1013904223)>>>0;
    const start=(seed>>>8)%9-1;
    const end=values[(seed>>>16)%values.length];
    run(`seed${i}`, () => [[start,end]], {cursor:!!(seed&1),x:(seed>>>24)%6,
      ...(seed&2 ? {selection:[1,4],column:!!(seed&4)} : {})});
  }
  return JSON.stringify(results);
}
