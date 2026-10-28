import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { resolve, relative, basename, sep } from 'node:path';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { runCommand } from '../quality-process.mjs';
import { startCaptureServer } from './server.mjs';

const sha256 = value => createHash('sha256').update(value).digest('hex');
export async function prepareBrowserCapture(context) {
  const { root, outputDirectory, caseId, signal } = context;
  const portable = value => relative(root, value).split(sep).join('/');
  const work = resolve(root, 'tmp/browser-mode-capture', `${caseId}-${basename(outputDirectory)}`);
  await mkdir(work, { recursive: true });
  const buildLog = resolve(outputDirectory, 'browser-build-slot.log');
  const argv = ['scripts/browser-capture/build.mjs', '--output', portable(work)];
  const result = await runCommand(process.execPath, argv, { cwd: root, env: process.env,
    log: buildLog, timeoutMs: 240000, signal });
  if (result.status !== 'passed') throw new Error(`Actual WASM/source build failed: ${portable(buildLog)}`);
  const build = JSON.parse(await readFile(resolve(work, 'build.json'), 'utf8'));
  if (build.status !== 'passed') throw new Error('Browser capture build has no passing receipt');
  const playwrightFile = process.env.PLAYWRIGHT_MODULE || resolve(root, 'source_repo/node_modules/playwright/index.mjs');
  const playwright = await import(pathToFileURL(playwrightFile));
  const artifacts = [...build.artifacts];
  const commands = [{ argv: [process.execPath, ...argv], exitCode: result.exitCode, log: portable(buildLog) },
    ...build.commands.map(command => ({ argv: command.argv, exitCode: command.exitCode, log: command.log }))];
  const server = await startCaptureServer(root, work);
  return { root, outputDirectory, caseId, signal, portable, playwright, server, build, artifacts, commands,
    async saveArtifact(name, bytes) {
      const file = resolve(outputDirectory, name);
      await writeFile(file, bytes);
      const entry = { path: portable(file), sha256: sha256(bytes) };
      artifacts.push(entry);
      return entry;
    } };
}

export async function capturePixels(page, host, name) {
  const png = await page.locator('.xterm').screenshot();
  const decoded = await page.evaluate(async bytes => {
    const image = await createImageBitmap(new Blob([Uint8Array.from(bytes)], { type: 'image/png' }));
    try {
      const canvas = document.createElement('canvas'); canvas.width = image.width; canvas.height = image.height;
      const context = canvas.getContext('2d');
      if (!context) throw new Error('No actual screenshot decoder canvas');
      context.drawImage(image, 0, 0);
      return { width: image.width, height: image.height,
        rgba: Array.from(context.getImageData(0, 0, image.width, image.height).data) };
    } finally { image.close(); }
  }, [...png]);
  await host.saveArtifact(`${name}.png`, png);
  const rgba = Buffer.from(decoded.rgba);
  const pixels = await host.saveArtifact(`${name}.rgba`, rgba);
  // The raw decoded bytes are retained. Comparing their digest compares every
  // channel without inflating a JSON receipt with millions of repeated numbers.
  return { width: decoded.width, height: decoded.height, rgbaSha256: pixels.sha256 };
}

export async function withBrowserPair(host, engine, scenario) {
  const browser = await host.playwright[engine].launch({ headless: true, timeout: 30000 });
  const captures = {};
  const evidence = { engine, runtimeVersion: browser.version(), pages: [] };
  const abort = () => { browser.close().catch(error => evidence.abortCleanupError = String(error)); };
  host.signal?.addEventListener('abort', abort, { once: true });
  try {
    for (const implementation of ['reference', 'rust']) {
      const context = await browser.newContext({ viewport: { width: 1000, height: 700 }, deviceScaleFactor: 1 });
      const page = await context.newPage(); page.setDefaultTimeout(20000);
      const pageEvidence = { implementation, errors: [], console: [] }; evidence.pages.push(pageEvidence);
      page.on('pageerror', error => pageEvidence.errors.push(error.stack ?? error.message));
      page.on('console', message => pageEvidence.console.push({ type: message.type(), text: message.text() }));
      try {
        await page.goto(`${host.server.origin}/${implementation}.html?implementation=${implementation}`);
        await page.waitForFunction(() => window.captureSession || window.captureInitializationError);
        await page.evaluate(async () => {
          if (window.captureInitializationError) throw new Error(window.captureInitializationError);
          await window.captureSession.ready;
        });
        captures[implementation] = await scenario(page, implementation);
        const disposed = await page.evaluate(() => window.captureSession.dispose());
        if (disposed.roots !== 0) throw new Error('Actual terminal dispose retained DOM roots');
        if (pageEvidence.errors.length) throw new Error(`Browser runtime errors: ${JSON.stringify(pageEvidence.errors)}`);
      } finally { await context.close(); }
    }
    evidence.status = 'passed';
    return { engine, runtimeVersion: evidence.runtimeVersion, implementation: 'rust-wasm', ...captures };
  } catch (error) {
    evidence.status = 'failed'; evidence.error = error.stack ?? String(error);
    throw error;
  } finally {
    host.signal?.removeEventListener('abort', abort);
    try { await host.saveArtifact(`${engine}-browser-evidence.json`, `${JSON.stringify(evidence, null, 2)}\n`); }
    finally { await browser.close(); }
  }
}
