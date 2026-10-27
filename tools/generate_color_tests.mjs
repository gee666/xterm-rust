// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Translates the literal assertion expressions without changing expected values.
import fs from 'node:fs';

const source = 'source_repo/src/common/Color.test.ts';
const input = fs.readFileSync(source, 'utf8');
const names = {
  'channels.toCss': 'channels::to_css', 'channels.toRgba': 'channels::to_rgba',
  'channels.toColor': 'channels::to_color', 'color.blend': 'value::blend',
  'color.opaque': 'value::opaque', 'color.isOpaque': 'value::is_opaque',
  'color.opacity': 'value::opacity', 'css.toColor': 'css::to_color',
  'rgb.relativeLuminance': 'rgb::relative_luminance', 'rgba.blend': 'rgba::blend',
  'rgba.ensureContrastRatio': 'rgba::ensure_contrast_ratio', 'rgba.toChannels': 'rgba::to_channels',
  'toPaddedHex': 'to_padded_hex', 'contrastRatio': 'contrast_ratio'
};

function translate(expression) {
  const tokens = expression.match(/0x[\da-f]+|\d*\.\d+|\d+|'[^']*'|[a-z_$][\w.$]*|[^\s]/gi) ?? [];
  let position = 0;
  function take(expected) {
    const token = tokens[position++];
    if (expected && token !== expected) throw new Error(`Expected ${expected}, found ${token}`);
    return token;
  }
  function node() {
    const token = take();
    if (token === '!') return `!${node()}`;
    if (token === 'undefined') return 'None';
    if (token === '[') {
      const entries = [];
      while (tokens[position] !== ']') { entries.push(node()); if (tokens[position] !== ']') take(','); }
      take(']'); return `[${entries.join(', ')}]`;
    }
    if (token === '{') {
      const fields = [];
      while (tokens[position] !== '}') {
        const key = take(); take(':'); const value = node();
        fields.push(`${key}: ${value}${key === 'css' ? '.into()' : ''}`);
        if (tokens[position] !== '}') take(',');
      }
      take('}'); return `Color { ${fields.join(', ')} }`;
    }
    if (token.startsWith("'")) return JSON.stringify(token.slice(1, -1));
    if (/^(?:0x[\da-f]+|\d*\.\d+|\d+)$/i.test(token)) return token.toLowerCase();
    const mapped = names[token];
    if (!mapped) throw new Error(`Unknown function ${token}`);
    take('(');
    let args = [];
    while (tokens[position] !== ')') { args.push(node()); if (tokens[position] !== ')') take(','); }
    take(')');
    if (token.startsWith('channels.')) {
      args[3] = args.length === 4 ? `Some(${args[3]})` : 'None';
    }
    if (token.startsWith('color.')) {
      args[0] = `&${args[0]}`;
      if (token === 'color.blend') args[1] = `&${args[1]}`;
      if (token === 'color.opacity' && !args[1].includes('.')) args[1] += '.0';
    }
    if (token === 'rgba.ensureContrastRatio' && !args[2].includes('.')) args[2] += '.0';
    if (token === 'contrastRatio') args = args.map(x => x.includes('.') ? x : `${x}.0`);
    let result = `${mapped}(${args.join(', ')})`;
    if (token === 'css.toColor') result += '.unwrap()';
    if (tokens[position] === '.' && tokens[position + 1] === 'toFixed') {
      take('.'); take('toFixed'); take('('); const precision = take(); take(')');
      result = `format!("{:.${precision}}", ${result})`;
    }
    return result;
  }
  const result = node();
  if (position !== tokens.length) throw new Error(`Unconsumed expression ${tokens.slice(position)}`);
  return result;
}

// Assertions in this upstream file are all single-line, literal expressions.
// Reject changed syntax rather than quietly losing a case during regeneration.
const lines = input.split(/\r?\n/);
const groups = [];
let current;
let assertions = 0;
for (let i = 0; i < lines.length; i++) {
  const line = lines[i];
  const test = line.match(/it\('([^']+)',/);
  if (test) { current = { name: test[1], assertions: [], line: i + 1 }; groups.push(current); }
  const assertion = line.match(/assert\.(equal|deepEqual|deepStrictEqual|ok)\((.*)\);/);
  if (!assertion) { if (line.includes('assert.')) throw new Error(`Unsupported assertion at line ${i + 1}`); continue; }
  if (!current) throw new Error('Assertion outside a test');
  const [, kind, expression] = assertion;
  let rust;
  if (kind === 'ok') rust = `assert!(${translate(expression)});`;
  else {
    // Split the outer argument pair, respecting nested arrays, objects and calls.
    let depth = 0, quote = false, split = -1;
    for (let c = 0; c < expression.length; c++) {
      if (expression[c] === "'") quote = !quote;
      if (quote) continue;
      if ('([{'.includes(expression[c])) depth++;
      if (')]}'.includes(expression[c])) depth--;
      if (expression[c] === ',' && depth === 0) { split = c; break; }
    }
    if (split < 0) throw new Error(`Cannot split assertion ${i + 1}`);
    const actual = translate(expression.slice(0, split));
    let expected = translate(expression.slice(split + 1));
    if (actual.startsWith('rgba::ensure_contrast_ratio') && expected !== 'None') expected = `Some(${expected})`;
    if (actual.startsWith('rgb::relative_luminance') || actual.startsWith('contrast_ratio')) {
      if (/^\d+$/.test(expected)) expected += '.0';
    }
    rust = `assert_eq!(${actual}, ${expected});`;
  }
  current.assertions.push(`    ${rust} // upstream line ${i + 1}`);
  assertions++;
}
const discovered = (input.match(/assert\./g) ?? []).length;
if (assertions !== discovered) throw new Error(`${assertions}/${discovered} assertions translated`);
const output = [
  '// Copyright (c) 2017 The xterm.js authors. All rights reserved.',
  '// Licensed under MIT. Generated by tools/generate_color_tests.mjs.',
  '// Faithful upstream fixture; exceeds 350 lines to retain all assertions in source order.',
  'use xterm_core::color::{channels, value, css, rgb, rgba, to_padded_hex, contrast_ratio, Color};', ''
];
for (const [index, group] of groups.entries()) {
  output.push(`// ${source}:${group.line}: ${group.name}`, '#[test]', `fn upstream_color_${index + 1}() {`, ...group.assertions, '}', '');
}
fs.mkdirSync('crates/xterm-core/tests', { recursive: true });
fs.writeFileSync('crates/xterm-core/tests/color_upstream.rs', output.join('\n'));
console.log(JSON.stringify({ source, tests: groups.length, assertions, discovered }, null, 2));
