// Independent source/Rust input construction. No font parser is implemented here.
import {withKern} from './font_metadata_scenarios.mjs';
export function mutationCases() {
  const changes = [
    ['Unchanged', 0, 0, []],
    ['IgnoreBadPairs', 0, 65535, [['kern', 12, [0, 1]]]],
    ['ReadBadPairs', 1, 65535, [['kern', 12, [0, 0]]]],
    ['AddPair', 0, 0, [['kern', 16, [0, 1]]]],
    ['RemovePairs', 0, 65535, [['kern', 16, [0, 0]]]],
    ['HeaderAlreadyRead', 0, 0, [['kern', 0, [0, 2]], ['kern', 4, [0, 0, 0, 0]]]],
    ['KernDescriptorAlreadyRead', 0, 0, [['directory:kern', 8, [255, 255, 255, 240]]]],
    ['GsubDescriptorAlreadyRead', 0, 0, [['directory:GSUB', 8, [255, 255, 255, 240]]]],
    ['CmapAlreadyRead', 0, 0, [['cmap', 2, [255, 255]]]],
    ['HeadAlreadyRead', 0, 0, [['head', 12, [0, 0, 0, 0]]]],
    ['BreakDeferredGsub', 0, 0, [['GSUB', 8, [255, 255]]]],
    ['DiscardCalt', 0, 0, [['calt', 0, [120, 120, 120, 120]]]],
    ['ThrowAfterMutation', 0, 0, [['kern', 16, [255, 255]]]],
  ];
  return ['SFNT', 'WOFF-raw', 'WOFF-kern'].flatMap(format => changes.map(([mode, coverage, pairs, writes]) =>
    ({name: `${format} ${mode}`, mode, format, coverage, pairs, writes, throws: mode === 'ThrowAfterMutation'})));
}
export function directory(bytes, tag) {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength), woff = view.getUint32(0) === 0x774f4646;
  const count = view.getUint16(woff ? 12 : 4), start = woff ? 44 : 12, width = woff ? 20 : 16;
  for (let index = 0; index < count; index++) {
    const at = start + index * width;
    if (String.fromCharCode(...bytes.slice(at, at + 4)) === tag) return {at, offset: view.getUint32(at + (woff ? 4 : 8)), woff};
  }
  throw new Error(`Missing input table ${tag}`);
}
export function mutationInput(base, item, compress) {
  const kern = new Uint8Array(24), header = new DataView(kern.buffer);
  header.setUint16(0, 1); header.setUint32(4, 2);
  header.setUint16(12, item.coverage); header.setUint16(16, item.pairs);
  const sfnt = withKern(base, kern);
  if (item.format === 'SFNT') return sfnt;
  const original = new DataView(sfnt.buffer), count = original.getUint16(4);
  const pieces = [], records = new Uint8Array(44 + count * 20), view = new DataView(records.buffer);
  records.set([119, 79, 70, 70]); records.set(sfnt.slice(0, 4), 4); view.setUint16(12, count);
  let cursor = records.length;
  for (let index = 0; index < count; index++) {
    const at = 12 + index * 16, record = 44 + index * 20;
    const offset = original.getUint32(at + 8), length = original.getUint32(at + 12);
    const data = sfnt.slice(offset, offset + length);
    const payload = item.format === 'WOFF-kern' && String.fromCharCode(...sfnt.slice(at, at + 4)) === 'kern' ? compress(data) : data;
    if (payload.length > data.length) throw new Error('Input kern must compress');
    records.set(sfnt.slice(at, at + 4), record);
    view.setUint32(record + 4, cursor); view.setUint32(record + 8, payload.length); view.setUint32(record + 12, length);
    pieces.push(payload); cursor += payload.length;
  }
  // Padding permits the same semantic payload writes on compressed source bytes.
  const output = new Uint8Array(cursor + 64); output.set(records);
  let at = records.length; for (const piece of pieces) {output.set(piece, at); at += piece.length;}
  new DataView(output.buffer).setUint32(8, output.length);
  return output;
}
export function mutate(bytes, item) {
  for (const [target, relative, data] of item.writes) {
    let at;
    if (target.startsWith('directory:')) {
      const record = directory(bytes, target.slice(10));
      at = record.at + relative - (record.woff ? 4 : 0);
    } else if (target === 'calt') {
      const table = directory(bytes, 'GSUB'), view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
      const features = table.offset + view.getUint16(table.offset + 6), count = view.getUint16(features);
      for (let index = 0; index < count; index++) {
        const record = features + 2 + index * 6;
        if (String.fromCharCode(...bytes.slice(record, record + 4)) === 'calt') {bytes.set(data, record); at = record;}
      }
      if (at === undefined) throw new Error('Input needs calt');
    } else at = directory(bytes, target).offset + relative;
    bytes.set(data, at);
  }
}
export async function bufferMutationScenarios(create, inputs) {
  const owner = globalThis.window ?? {}, results = {};
  const descriptors = [[globalThis, 'window'], [globalThis, 'navigator'], [owner, 'queryLocalFonts'], [console, 'warn']]
    .map(([object, key]) => [object, key, Object.getOwnPropertyDescriptor(object, key)]);
  if (!globalThis.window) Object.defineProperty(globalThis, 'window', {configurable: true, value: owner});
  Object.defineProperty(globalThis, 'navigator', {configurable: true, value: {}});
  let current;
  Object.defineProperty(owner, 'queryLocalFonts', {configurable: true, value: async () => mutationCases().map(item =>
    ({family: item.name, async blob() {return {async arrayBuffer() {return current.buffer;}};}}))});
  try {
    for (const [index, item] of mutationCases().entries()) {
      current = inputs[index].slice();
      let join, complete, refreshes = 0, warnings = 0;
      const term = {rows: 4, element: {style: {}}, options: {logLevel: 'info', get fontFamily() {
        if (complete) {const done = complete; complete = undefined; done();} return item.name;
      }}, registerCharacterJoiner(callback) {join = callback; return 0;}, deregisterCharacterJoiner() {}, refresh() {refreshes++;}};
      const sentinel = {};
      Object.defineProperty(console, 'warn', {configurable: true, value() {warnings++; mutate(current, item); if (item.throws) throw sentinel;}});
      const addon = create({fallbackLigatures: ['===']});
      try {
        addon.activate(term); const initial = join('a===b');
        await new Promise(resolve => {complete = resolve;});
        results[item.name] = {initial, ranges: join('a===b'), warnings, refreshes, cached: join('a===b')};
      } finally {addon.dispose();}
    }
    return results;
  } finally {
    for (const [object, key, descriptor] of descriptors.reverse()) {
      if (descriptor) Object.defineProperty(object, key, descriptor); else delete object[key];
    }
  }
}
