// Shared inputs and recorded host callbacks, never a font parser.
export function buildHintFont(base, change) {
  const source = new DataView(base.buffer, base.byteOffset, base.byteLength);
  const records = [];
  for (let i = 0; i < source.getUint16(4); i++) {
    const at = 12 + i * 16, tag = String.fromCharCode(...base.subarray(at, at + 4));
    if (tag === change.tag) continue;
    const start = source.getUint32(at + 8), length = source.getUint32(at + 12);
    const data = base.slice(start, start + length);
    if (tag === 'head' && change.badHead) data[12] ^= 1;
    records.push({tag, data, length});
  }
  for (const extra of change.entries) {
    const record = {...extra, tag: change.tag, data: new Uint8Array(extra.data)};
    if (extra.first) records.unshift(record); else records.push(record);
  }
  if (change.tail) records.push({tag:'zzzz', data:new Uint8Array(change.tail), length:change.tail.length});
  const woff = change.woff, header = woff ? 44 : 12, width = woff ? 20 : 16;
  const output = new Uint8Array(header + records.length * width + records.reduce((n, r) => n + r.data.length, 0));
  const view = new DataView(output.buffer);
  if (woff) {
    output.set([119,79,70,70]); output.set(base.subarray(0,4),4); view.setUint16(12, records.length);
    view.setUint32(8, output.length);
  } else { output.set(base.subarray(0,12)); view.setUint16(4, records.length); }
  let cursor = header + records.length * width;
  for (let i = 0; i < records.length; i++) {
    const r = records[i], at = header + i * width;
    output.set([...r.tag].map(c => c.charCodeAt(0)), at);
    view.setUint32(at + (woff ? 4 : 8), r.offset ?? cursor);
    if (woff) view.setUint32(at + 8, r.compressedLength ?? r.data.length);
    view.setUint32(at + 12, r.length ?? r.data.length);
    output.set(r.data, cursor); cursor += r.data.length;
  }
  return output;
}

export async function hintAddonScenarios(create, base, cases) {
  const nav = Object.getOwnPropertyDescriptor(globalThis, 'navigator');
  const win = Object.getOwnPropertyDescriptor(globalThis, 'window'), owner = globalThis.window ?? {};
  if (!win) Object.defineProperty(globalThis, 'window', {configurable:true,value:owner});
  const query = Object.getOwnPropertyDescriptor(owner, 'queryLocalFonts');
  let family, join, complete, reads = 0, refreshes = 0;
  const terminal = {rows:5,element:{style:{}},options:{logLevel:'info',
    get fontFamily() { if (complete) { const done = complete; complete = undefined; done(); } return family; }},
    registerCharacterJoiner(callback) {join = callback; return 0;},
    deregisterCharacterJoiner() {}, refresh() {refreshes++;}};
  let addon;
  try {
    Object.defineProperty(globalThis, 'navigator', {configurable:true,value:{}});
    Object.defineProperty(owner, 'queryLocalFonts', {configurable:true,value:async () => cases.map((c, i) => ({
      family:'HintTable' + i, async blob() { reads++; return {async arrayBuffer() {return buildHintFont(base,c).buffer;}};}
    }))});
    addon = create({fallbackLigatures:['===']}); family = 'HintTable0'; addon.activate(terminal);
    const output = [];
    for (let i = 0; i < cases.length; i++) {
      family = 'HintTable' + i;
      const before = {reads,refreshes}, loading = join('===');
      await new Promise(resolve => {complete = resolve;});
      const ranges = join('===');
      join('==='); join('===');
      output.push({name:cases[i].name,loading,ranges,reads:reads-before.reads,refreshes:refreshes-before.refreshes});
    }
    return output;
  } finally {
    if (addon) addon.dispose();
    if (nav) Object.defineProperty(globalThis, 'navigator', nav); else delete globalThis.navigator;
    if (query) Object.defineProperty(owner, 'queryLocalFonts', query); else delete owner.queryLocalFonts;
    if (!win) delete globalThis.window;
  }
}
