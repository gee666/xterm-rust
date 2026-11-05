// The same addon-boundary operations execute against source and Rust/WASM.
export async function svgLoadingScenario(create, bytes) {
  const bad = bytes.slice(), view = new DataView(bad.buffer);
  let changed = false;
  for (let i = 0; i < view.getUint16(4); i++) {
    const at = 12 + i * 16;
    if (view.getUint32(at) === 0x44534947) {
      view.setUint32(at, 0x53564720);
      view.setUint32(at + 8, bad.length - 1);
      view.setUint32(at + 12, 1);
      bad[bad.length - 1] = 0;
      changed = true;
      break;
    }
  }
  if (!changed) throw new Error('Expected ignored DSIG descriptor in source fixture');
  const navigatorDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'navigator');
  const windowDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'window');
  const owner = globalThis.window ?? {};
  const queryDescriptor = Object.getOwnPropertyDescriptor(owner, 'queryLocalFonts');
  let reads = 0, refreshes = 0, join, completed, family = 'BadSvg';
  const terminal = {rows:5,element:{style:{}},options:{logLevel:'info',get fontFamily(){
    if (completed) {const done=completed;completed=undefined;done();} return family;
  }},registerCharacterJoiner(callback){join=callback;return 0;},deregisterCharacterJoiner(){},refresh(){refreshes++;}};
  Object.defineProperty(globalThis, 'navigator', {configurable:true,value:{}});
  if (!windowDescriptor) Object.defineProperty(globalThis, 'window', {configurable:true,value:owner});
  Object.defineProperty(owner, 'queryLocalFonts', {configurable:true,value:async()=>
    [['BadSvg',bad],['GoodSvg',bytes]].map(([family,data])=>({family,async blob(){reads++;return {async arrayBuffer(){return data.buffer.slice(data.byteOffset,data.byteOffset+data.byteLength);}};}}))});
  const addon=create({fallbackLigatures:['===']});
  try {
    addon.activate(terminal);
    const loading=join('===');
    await new Promise(resolve=>{completed=resolve;});
    const rejected={ranges:join('==='),reads,refreshes};
    join('===');join('===');
    const cached={reads,refreshes};
    family='GoodSvg';
    const retry=join('===');
    await new Promise(resolve=>{completed=resolve;});
    return {loading,rejected,cached,retry,accepted:{ranges:join('==='),reads,refreshes}};
  } finally {
    addon.dispose();
    if(navigatorDescriptor)Object.defineProperty(globalThis,'navigator',navigatorDescriptor);else delete globalThis.navigator;
    if(queryDescriptor)Object.defineProperty(owner,'queryLocalFonts',queryDescriptor);else delete owner.queryLocalFonts;
    if(!windowDescriptor)delete globalThis.window;
  }
}
