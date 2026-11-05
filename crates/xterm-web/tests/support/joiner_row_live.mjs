export function liveRowFixture(mode, onRead) {
  const marker={};
  const trace=[];
  let converted=false;
  const range={
    get 0() { if(converted) {trace.push('row:start'); return mode==='unmatched' ? 9 : 1;} return 1; },
    set 0(_) {},
    get 1() {
      if(converted) {
        trace.push('row:end');
        onRead?.();
        if(mode==='unmatched' || mode==='error') throw marker;
        return mode==='fractional' ? 2.5 : 4;
      }
      return 3;
    },
    set 1(_) {converted=true;}
  };
  return {marker,trace,handler() {converted=false;return [range];}};
}
