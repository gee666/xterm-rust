// Licensed under MIT. Keep source method bodies intact in the browser oracle.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
const root = new URL('../', import.meta.url);
const methods = [
  ['src/common/services/CoreService.ts', 'public triggerDataEvent(data: string, wasUserInput: boolean = false): void {', 'triggerDataEvent'],
  ['src/common/services/LogService.ts', 'private _evalLazyOptionalParams(optionalParams: any[]): void {', 'evalLazy'],
  ['src/common/services/LogService.ts', 'private _log(type: LogType, message: string, optionalParams: any[]): void {', 'log'],
  ['src/common/services/LogService.ts', 'public trace(message: string, ...optionalParams: any[]): void {', 'trace'],
  ['src/common/services/LogService.ts', 'public debug(message: string, ...optionalParams: any[]): void {', 'debug'],
];
for (const [path, signature, name] of methods) {
  test(`${name} body matches unchanged reference`, async () => {
    const source = await readFile(new URL(`source_repo/${path}`, root), 'utf8');
    const normalized = source.replaceAll('\r\n', '\n');
    const start = normalized.indexOf(signature);
    assert.ok(start >= 0);
    const from = start + signature.length;
    const end = normalized.indexOf('\n  }', from);
    assert.ok(end > from);
    const fixture = await readFile(new URL(`crates/xterm-web/tests/fixtures/raw-input-${name}.js`, root), 'utf8');
    assert.equal(fixture.replaceAll('\r\n', '\n').trim(), normalized.slice(from, end).trim());
    console.log(`${path} sha256=${createHash('sha256').update(source).digest('hex')}`);
    console.log(`${name} sha256=${createHash('sha256').update(fixture).digest('hex')}`);
  });
}
