// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Execute all original Params.test.ts assertions with actual Mocha and Chai.
import fs from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { stripTypeScriptTypes } from 'node:module';
import { createHash } from 'node:crypto';

const temporary = path.resolve('tmp/params-baseline');
const dependencies = path.resolve(process.env.PARAMS_TEST_DEPS ?? 'tmp/color-oracle/node_modules');
await fs.mkdir(temporary, { recursive: true });
await fs.mkdir('results/params', { recursive: true });
const sourcePath = 'source_repo/src/common/parser/Params.ts';
const testPath = 'source_repo/src/common/parser/Params.test.ts';
const source = await fs.readFile(sourcePath, 'utf8');
const originalTests = await fs.readFile(testPath, 'utf8');
const implementation = stripTypeScriptTypes(source.replace(/^import \{ IParams, ParamsArray \} from '\.\/Types';\r?\n/m, ''), { mode: 'transform' });
const implementationPath = path.join(temporary, 'Params.mjs');
await fs.writeFile(implementationPath, implementation);
const mochaUrl = pathToFileURL(path.join(dependencies, 'mocha/index.js')).href;
const chaiUrl = pathToFileURL(path.join(dependencies, 'chai/index.js')).href;
const testModule = stripTypeScriptTypes(originalTests.replace(/^import \{ ParamsArray \} from '\.\/Types';\r?\n/m, ''), { mode: 'transform' })
  .replace("from 'chai'", `from ${JSON.stringify(chaiUrl)}`)
  .replace("from './Params'", `from ${JSON.stringify(pathToFileURL(implementationPath).href)}`);
const testModulePath = path.join(temporary, 'Params.test.mjs');
await fs.writeFile(testModulePath, testModule);
const { default: Mocha } = await import(mochaUrl);
const mocha = new Mocha({ reporter: 'spec' });
mocha.addFile(testModulePath);
await mocha.loadFilesAsync();
let runner;
const failures = await new Promise(resolve => { runner = mocha.run(resolve); });
const summary = {
  sourcePath, testPath,
  sourceSha256: createHash('sha256').update(source).digest('hex'),
  testSha256: createHash('sha256').update(originalTests).digest('hex'),
  node: process.version, platform: process.platform,
  mocha: JSON.parse(await fs.readFile(path.join(dependencies, 'mocha/package.json'), 'utf8')).version,
  chai: JSON.parse(await fs.readFile(path.join(dependencies, 'chai/package.json'), 'utf8')).version,
  discovered: runner.stats.tests, executed: runner.stats.tests - runner.stats.pending,
  passed: runner.stats.passes, failed: failures, skipped: runner.stats.pending, unrun: 0
};
await fs.writeFile('results/params/original.json', JSON.stringify(summary, null, 2) + '\n');
console.log(JSON.stringify(summary, null, 2));
if (failures) process.exitCode = 1;
