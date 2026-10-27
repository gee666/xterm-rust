// Copyright (c) 2019-2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Shared read-only upstream streaming-parser test runner.
import assert from 'node:assert/strict';
import { stripTypeScriptTypes } from 'node:module';
import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import vm from 'node:vm';

export async function parserOracle(name) {
  const paths = [
    'source_repo/src/common/parser/Constants.ts',
    'source_repo/src/common/input/TextDecoder.ts',
    'source_repo/src/common/StringBuilder.ts',
    ...(name === 'Dcs' ? ['source_repo/src/common/parser/Params.ts'] : []),
    `source_repo/src/common/parser/${name}Parser.ts`,
    `source_repo/src/common/parser/${name}Parser.test.ts`,
  ];
  const texts = await Promise.all(paths.map(path => readFile(path, 'utf8')));
  // Remove only module boundaries and type syntax. Runtime implementation and
  // every upstream assertion execute unchanged inside a single realm.
  const erase = text => stripTypeScriptTypes(
    text.replace(/^import .*;\r?$/gm, '').replace(/^export /gm, ''), { mode: 'transform' });
  const suites = [];
  const tests = [];
  const context = vm.createContext({ assert,
    describe(name, callback) {
      suites.push({ name, before: [], after: [] }); callback(); suites.pop();
    },
    beforeEach(callback) { suites.at(-1).before.push(callback); },
    afterEach(callback) { suites.at(-1).after.push(callback); },
    it(name, callback) {
      tests.push({ name: [...suites.map(s => s.name), name].join(' / '), callback,
        before: suites.flatMap(s => s.before), after: suites.toReversed().flatMap(s => s.after) });
    },
  });
  vm.runInContext(texts.slice(0, -1).map(erase).join('\n') + `\nglobalThis.Parser = ${name}Parser;` +
    (name === 'Dcs' ? '\nglobalThis.Params = Params;' : ''), context);
  vm.runInContext(erase(texts.at(-1)), context);
  const outcomes = [];
  for (const test of tests) {
    try {
      for (const before of test.before) await before();
      await test.callback.call({ timeout() {} });
      outcomes.push({ name: test.name, status: 'passed' });
    } catch (error) {
      outcomes.push({ name: test.name, status: 'failed', error: String(error.stack) });
    } finally {
      for (const after of test.after) await after();
    }
  }
  return {
    context,
    resolvedPromise(callback) {
      return vm.runInContext('Promise.resolve()', context).then(callback);
    },
    baseline: { discovered: outcomes.length, passed: outcomes.filter(x => x.status === 'passed').length,
      failed: outcomes.filter(x => x.status === 'failed').length, skipped: 0, outcomes },
    sources: paths.map((path, index) => ({ path, sha256: createHash('sha256').update(texts[index]).digest('hex') })),
    utf32String(data) {
      return vm.runInContext(`utf32ToString(Uint32Array.from([${data.join(',')}]))`, context);
    },
  };
}

export const utf16Hex = data => Array.from({ length: data.length }, (_, i) =>
  data.charCodeAt(i).toString(16)).join(',');
