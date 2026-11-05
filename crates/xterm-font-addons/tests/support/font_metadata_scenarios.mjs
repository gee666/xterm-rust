// Shared source/Rust addon boundary input. This host records callbacks only.
export async function metadataRejectionScenarios(create, bytes) {
  const bad = bytes.slice(), view = new DataView(bad.buffer, bad.byteOffset, bad.byteLength);
  for (let index = 0; index < view.getUint16(4); index++) {
    const record = 12 + index * 16;
    if (view.getUint32(record) === 0x68656164) { bad[view.getUint32(record + 8) + 12] ^= 1; break; }
  }
  const nav = Object.getOwnPropertyDescriptor(globalThis, 'navigator');
  const windowDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'window');
  const owner = globalThis.window ?? {};
  if (!windowDescriptor) Object.defineProperty(globalThis, 'window', { configurable: true, value: owner });
  const query = Object.getOwnPropertyDescriptor(owner, 'queryLocalFonts');
  let reads = 0, refreshes = 0, join, complete, family = 'BadMetadata';
  const term = { rows: 5, element: { style: {} }, options: { logLevel: 'info',
    get fontFamily() { if (complete) { const done = complete; complete = undefined; done(); } return family; }
  }, registerCharacterJoiner(callback) { join = callback; return 0; },
  deregisterCharacterJoiner() {}, refresh() { refreshes++; } };
  Object.defineProperty(globalThis, 'navigator', { configurable: true, value: {} });
  Object.defineProperty(owner, 'queryLocalFonts', { configurable: true, value: async () =>
    ['BadMetadata', 'GoodMetadata'].map(family => ({ family, async blob() {
      reads++; const data = family === 'BadMetadata' ? bad : bytes;
      return { async arrayBuffer() { return data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength); } };
    } })) });
  const addon = create({ fallbackLigatures: ['==='] });
  try {
    addon.activate(term);
    const loading = join('===');
    await new Promise(resolve => { complete = resolve; });
    const rejected = { ranges: join('==='), reads, refreshes };
    join('==='); join('===');
    const cachedFailure = { reads, refreshes };
    family = 'GoodMetadata';
    const retry = join('===');
    await new Promise(resolve => { complete = resolve; });
    return { loading, rejected, cachedFailure, retry, accepted: { ranges: join('==='), reads, refreshes } };
  } finally {
    addon.dispose();
    if (nav) Object.defineProperty(globalThis, 'navigator', nav); else delete globalThis.navigator;
    if (query) Object.defineProperty(owner, 'queryLocalFonts', query); else delete owner.queryLocalFonts;
    if (!windowDescriptor) delete globalThis.window;
  }
}
