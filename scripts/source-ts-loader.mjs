// Copyright (c) 2019-2026 The xterm.js authors. All rights reserved. MIT license.
// Shared in-memory loader. No source files, imports or test assertions are edited.
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
export function sourceLoader(root = process.cwd()) {
  const source = path.join(root, 'source_repo');
  const require = createRequire(path.join(source, 'package.json'));
  const ts = require('typescript');
  const cache = new Map();
  const hashes = new Map();
  function load(filename) {
    if (cache.has(filename)) return cache.get(filename).exports;
    const text = fs.readFileSync(filename, 'utf8');
    hashes.set(path.relative(root, filename).replaceAll('\\', '/'), createHash('sha256').update(text).digest('hex'));
    const module = { exports: {} }; cache.set(filename, module);
    const code = ts.transpileModule(text, { compilerOptions: { target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.CommonJS, experimentalDecorators: true, esModuleInterop: false }}).outputText;
    const localRequire = id => {
      const candidate = id.startsWith('.') ? path.resolve(path.dirname(filename), id)
        : /^(common|browser|headless)\//.test(id) ? path.join(source, 'src', id) : undefined;
      if (candidate && fs.existsSync(candidate + '.ts')) return load(candidate + '.ts');
      return require(id);
    };
    new Function('require', 'module', 'exports', code)(localRequire, module, module.exports);
    return module.exports;
  }
  return { source, load, hashes, typescript: ts.version };
}
