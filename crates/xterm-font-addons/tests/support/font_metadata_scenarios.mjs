// Shared source/Rust addon boundary input. This host records callbacks only.
export function withKern(bytes, payload) {
  // Insert a directory entry without changing any original table payload.
  const original = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const count = original.getUint16(4), directoryEnd = 12 + count * 16;
  const bad = new Uint8Array(bytes.length + 16 + payload.length);
  bad.set(bytes.subarray(0, directoryEnd));
  bad.set(bytes.subarray(directoryEnd), directoryEnd + 16);
  const view = new DataView(bad.buffer);
  view.setUint16(4, count + 1);
  for (let i = 0; i < count; i++)view.setUint32(12 + i * 16 + 8, original.getUint32(12 + i * 16 + 8) + 16);
  view.setUint32(directoryEnd, 0x6b65726e);
  view.setUint32(directoryEnd + 8, bytes.length + 16);
  view.setUint32(directoryEnd + 12, payload.length);
  bad.set(payload, bytes.length + 16);
  return bad;
}
export async function metadataRejectionScenarios(create, bytes) {
  const bad = bytes.slice(), view = new DataView(bad.buffer, bad.byteOffset, bad.byteLength);
  for (let index = 0; index < view.getUint16(4); index++) {
    const record = 12 + index * 16;
    if (view.getUint32(record) === 0x68656164) { bad[view.getUint32(record + 8) + 12] ^= 1; break; }
  }
  const badKern = withKern(bytes, new Uint8Array([0, 2]));
  const nav = Object.getOwnPropertyDescriptor(globalThis, 'navigator');
  const windowDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'window');
  const owner = globalThis.window ?? {};
  if (!windowDescriptor) Object.defineProperty(globalThis, 'window', { configurable: true, value: owner });
  const query = Object.getOwnPropertyDescriptor(owner, 'queryLocalFonts');
  let reads = 0, refreshes = 0, join, complete, family;
  const term = { rows: 5, element: { style: {} }, options: { logLevel: 'info',
    get fontFamily() { if (complete) { const done = complete; complete = undefined; done(); } return family; }
  }, registerCharacterJoiner(callback) { join = callback; return 0; },
  deregisterCharacterJoiner() {}, refresh() { refreshes++; } };
  Object.defineProperty(globalThis, 'navigator', { configurable: true, value: {} });
  // The source catalog is cached for the whole module. Both cases must share
  // one catalog, while different family keys isolate their load-result caches.
  Object.defineProperty(owner, 'queryLocalFonts', { configurable: true, value: async () =>
    [['BadMetadata', bad], ['GoodMetadata', bytes], ['BadKern', badKern], ['GoodKern', bytes]].map(([family, data]) => ({ family, async blob() {
      reads++;
      return { async arrayBuffer() { return data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength); } };
    } })) });
  const addon = create({ fallbackLigatures: ['==='] });
  async function scenario(badFamily, goodFamily) {
    const initialReads = reads, initialRefreshes = refreshes;
    const counters = () => ({ reads: reads - initialReads, refreshes: refreshes - initialRefreshes });
    family = badFamily;
    const loading = join('===');
    await new Promise(resolve => { complete = resolve; });
    const rejected = { ranges: join('==='), ...counters() };
    join('==='); join('===');
    const cachedFailure = counters();
    family = goodFamily;
    const retry = join('===');
    await new Promise(resolve => { complete = resolve; });
    return { loading, rejected, cachedFailure, retry, accepted: { ranges: join('==='), ...counters() } };
  }
  try {
    family = 'BadMetadata';
    addon.activate(term);
    return { metadata: await scenario('BadMetadata', 'GoodMetadata'), kern: await scenario('BadKern', 'GoodKern') };
  } finally {
    addon.dispose();
    if (nav) Object.defineProperty(globalThis, 'navigator', nav); else delete globalThis.navigator;
    if (query) Object.defineProperty(owner, 'queryLocalFonts', query); else delete owner.queryLocalFonts;
    if (!windowDescriptor) delete globalThis.window;
  }
}
