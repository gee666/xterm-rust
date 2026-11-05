// Forced collection runs between page jobs so WeakRef's keep-alive rule expires.
import assert from 'node:assert/strict';

async function collect(session) {
  // Two full collections are a fixed test protocol, not a retry-until-pass loop.
  await session.send('HeapProfiler.collectGarbage');
  await session.send('HeapProfiler.collectGarbage');
}

async function cases(page, session, entry) {
  function check(name, actual, expected) {
    const result = { name, actual, status: 'failed' }; entry.cases.push(result);
    assert.deepEqual(actual, expected);
    result.status = 'passed';
  }
  await page.evaluate(async () => {
    const terminal = new Terminal({ cols: 12, rows: 4 });
    const addon = new SerializeAddon();
    terminal.loadAddon(addon);
    await new Promise(resolve => terminal.write('held', resolve));
    globalThis.serializationGc = { addon, terminal: new WeakRef(terminal), weakAddon: new WeakRef(addon) };
  });
  await collect(session);
  const retained = await page.evaluate(() => ({
    alive: !!serializationGc.terminal.deref(), ansi: serializationGc.addon.serialize()
  }));
  check('retained host', retained, { alive: true, ansi: 'held' });
  await page.evaluate(() => serializationGc.addon.dispose());
  await collect(session);
  const disposed = await page.evaluate(() => ({
    alive: !!serializationGc.terminal.deref(), ansi: serializationGc.addon.serialize()
  }));
  check('disposed addon retains host', disposed, { alive: true, ansi: 'held' });
  await page.evaluate(() => { delete serializationGc.addon; });
  await collect(session);
  const released = await page.evaluate(() => ({
    terminal: serializationGc.terminal.deref() === undefined,
    addon: serializationGc.weakAddon.deref() === undefined
  }));
  check('released addon collects', released, { terminal: true, addon: true });
  await page.evaluate(() => {
    const terminal = new Terminal({ cols: 12, rows: 4 });
    const addon = new SerializeAddon();
    terminal.loadAddon(addon);
    globalThis.serializationCycle = { terminal: new WeakRef(terminal), addon: new WeakRef(addon) };
  });
  await collect(session);
  const cycle = await page.evaluate(() => ({
    terminal: serializationCycle.terminal.deref() === undefined,
    addon: serializationCycle.addon.deref() === undefined
  }));
  check('unreachable loaded cycle collects', cycle, { terminal: true, addon: true });
  return { retained, disposed, released, cycle, collections: 8, cases: 4 };
}

export async function serializationGc(playwright, origins) {
  const report = { status: 'failed', browser: 'chromium', engines: [], unsupported: [
    { browser: 'firefox', status: 'unrun', reason: 'This runner has no forced-GC protocol for Firefox.' },
    { browser: 'webkit', status: 'unrun', reason: 'This runner has no forced-GC protocol for WebKit.' }
  ] };
  let browser;
  try {
    browser = await playwright.chromium.launch({ headless: true, timeout: 30000 });
    report.version = browser.version();
    for (const [engine, origin] of origins) {
      const entry = { engine, status: 'failed', cases: [] }; report.engines.push(entry);
      const page = await browser.newPage(); page.setDefaultTimeout(15000);
      let session, deadline;
      try {
        await page.goto(`${origin}/test`);
        await page.waitForFunction(() => typeof Terminal === 'function' && typeof SerializeAddon === 'function');
        session = await page.context().newCDPSession(page);
        await session.send('HeapProfiler.enable');
        entry.result = await Promise.race([cases(page, session, entry), new Promise((_, reject) => {
          deadline = setTimeout(() => reject(Error(`${engine} GC protocol exceeded30s`)), 30000);
        })]);
        entry.status = 'passed';
      } catch (error) { entry.error = String(error.stack ?? error); }
      finally {
        clearTimeout(deadline);
        try { if (session) await session.detach(); } finally { await page.close(); }
      }
    }
    if (report.engines.every(entry => entry.status === 'passed')) report.status = 'passed';
  } finally { if (browser) await browser.close(); }
  const executed = report.engines.flatMap(entry => entry.cases);
  report.counts = { discovered: 24, executed: executed.length,
    passed: executed.filter(entry => entry.status === 'passed').length,
    failed: executed.filter(entry => entry.status === 'failed').length,
    skipped: 0, unrun: 24 - executed.length };
  return report;
}
