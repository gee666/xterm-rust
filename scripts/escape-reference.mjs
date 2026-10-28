// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Load unchanged upstream modules in one JavaScript realm.
import { readFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import vm from 'node:vm';
import { eraseModuleBoundaries } from './parser-oracle-common.mjs';

export async function escapeReference() {
  const modules = [
    ['parser/Constants.ts', 'ParserState, ParserAction, ParserConstants, OscState'],
    ['parser/Types.ts', 'ParserStackType'],
    ['Lifecycle.ts', 'Disposable, toDisposable'],
    ['input/TextDecoder.ts', 'utf32ToString'],
    ['StringBuilder.ts', 'LimitedStringBuilder'],
    ['parser/Params.ts', 'Params'],
    ['parser/OscParser.ts', 'OscParser'],
    ['parser/DcsParser.ts', 'DcsParser'],
    ['parser/ApcParser.ts', 'ApcParser'],
    ['parser/EscapeSequenceParser.ts', 'EscapeSequenceParser, VT500_TRANSITION_TABLE'],
  ];
  const context = vm.createContext({});
  const sources = [];
  for (const [relative, exports] of modules) {
    const path = `source_repo/src/common/${relative}`;
    const source = await readFile(path, 'utf8');
    sources.push({ path, sha256: createHash('sha256').update(source).digest('hex') });
    // An IIFE preserves each original module's private declarations. In
    // particular, OSC/DCS/APC each have a private EMPTY_HANDLERS constant.
    vm.runInContext(`Object.assign(globalThis, (() => {\n${eraseModuleBoundaries(source)}\nreturn {${exports}};\n})());`, context, { filename: path });
  }
  return {
    context, sources,
    resolvedPromise(callback) { return vm.runInContext('Promise.resolve()', context).then(callback); },
    utf32String(data) {
      context.input = Uint32Array.from(data);
      const result = vm.runInContext('utf32ToString(input)', context);
      delete context.input;
      return result;
    },
  };
}
