// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
// Execute unchanged RectangleRenderer.ts. A GL recorder observes uploaded data;
// real WebGL2 pixel assertions live in wasm_webgl_rectangles.rs.
import {readFile, writeFile} from 'node:fs/promises';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import path from 'node:path';
import vm from 'node:vm';
const sourceRoot = path.resolve(process.env.SOURCE_ROOT ?? 'source_repo');
const require = createRequire(path.join(sourceRoot, 'package.json'));
const esbuild = require('esbuild');
const file = path.join(sourceRoot, 'addons/addon-webgl/src/RectangleRenderer.ts');
const built = await esbuild.build({entryPoints: [file], bundle: true, write: false, format: 'cjs',
  platform: 'node', metafile: true, alias: {browser: path.join(sourceRoot, 'src/browser'), common: path.join(sourceRoot, 'src/common')}});
const module = {exports: {}};
vm.runInNewContext(built.outputFiles[0].text, {module, exports: module.exports, Float32Array, Uint8Array, console}, {filename: file});
const {RectangleRenderer} = module.exports;
const inputs = [];
for (const input of Object.keys(built.metafile.inputs).sort()) {
  inputs.push({path: path.relative(sourceRoot, input).replaceAll('\\', '/'),
    sha256: createHash('sha256').update(await readFile(input)).digest('hex')});
}
function glRecorder() {
  const gl = {ARRAY_BUFFER: 1, ELEMENT_ARRAY_BUFFER: 2, STATIC_DRAW: 3, DYNAMIC_DRAW: 4,
    FLOAT: 5, TRIANGLE_STRIP: 6, UNSIGNED_BYTE: 7, VERTEX_SHADER: 8, FRAGMENT_SHADER: 9,
    COMPILE_STATUS: 10, LINK_STATUS: 11,
    createProgram: () => ({}), createShader: () => ({}), createVertexArray: () => ({}), createBuffer: () => ({}),
    getProgramParameter: () => true, getShaderParameter: () => true, getUniformLocation: () => ({}),
    bufferData(target, values) { if (target === this.ARRAY_BUFFER) this.last = [...values]; },
    drawElementsInstanced(_mode, _count, _type, _offset, count) { this.count = count; }};
  for (const name of ['attachShader','shaderSource','compileShader','linkProgram','deleteProgram',
    'deleteShader','deleteVertexArray','deleteBuffer','bindVertexArray','bindBuffer','enableVertexAttribArray',
    'vertexAttribPointer','vertexAttribDivisor','useProgram','uniformMatrix4fv']) gl[name] = () => {};
  return gl;
}
const bits = value => new Uint32Array(new Float32Array([value]).buffer)[0];
const colors = [0xa1b2c3ff, 0x12345680, 0xeeddcc40];
const palette = Array.from({length: 256}, (_, i) => (((i * 971653) << 8) | (i ^ 0xaa)) >>> 0);
const cases = [];
function run(name, cols, rows, cell, canvas, cells, cursor) {
  const gl = glRecorder();
  const sourceColors = {foreground: {rgba: colors[0]}, background: {rgba: colors[1]}, cursor: {rgba: colors[2]},
    ansi: palette.map(rgba => ({rgba}))};
  const theme = {colors: sourceColors, onChangeColors: () => ({dispose() {}})};
  const dimensions = {device: {cell: {width: cell[0], height: cell[1]}, canvas: {width: canvas[0], height: canvas[1]}}};
  const renderer = new RectangleRenderer({cols, rows}, gl, dimensions, theme, {error(message) {throw Error(message);}});
  const model = {cells: new Uint32Array(cells.flatMap(([fg,bg]) => [0,bg,fg,0])), cursor};
  renderer.handleResize();
  renderer.updateBackgrounds(model);
  renderer.renderBackgrounds();
  const background = {attributes: gl.last.map(bits), count: gl.count};
  renderer.updateCursor(model);
  renderer.renderCursor();
  const expectedCursor = {attributes: gl.last.map(bits), count: gl.count};
  renderer.dispose();
  cases.push({name, cols, rows, cell, canvas, cells, cursor: cursor ?? null, background, expectedCursor});
}
const modes = [0, 0x1000000, 0x2000000, 0x3000000];
for (let inverse = 0; inverse < 2; inverse++) for (const mode of modes) {
  const fg = mode | 37 | (inverse ? 0x4000000 : 0);
  run(`color-${inverse}-${mode}`, 4, 1, [7.25, 13.5], [29, 13.5],
    [[fg,mode | 52],[fg,mode | 52],[fg,0],[0,0]], undefined);
}
for (const style of ['block','bar','underline','outline','none']) {
  run(`cursor-${style}`, 5, 3, [9.3, 17.75], [46.5, 53.25], Array.from({length: 15}, () => [0,0]),
    {x: 2, y: 1, width: 2, style, cursorWidth: 3, dpr: 1.25});
}
let seed = 0x57e61a23;
function random() { seed ^= seed << 13; seed ^= seed >>> 17; seed ^= seed << 5; return seed >>> 0; }
for (let i = 0; i < 128; i++) {
  const cols = random() % 48 + 1, rows = random() % 5 + 1;
  const cell = [(random() % 80 + 1) / 4, (random() % 100 + 1) / 4];
  const cells = Array.from({length: cols * rows}, () => {
    const fgMode = modes[random() % 4], bgMode = modes[random() % 4];
    const fg = fgMode | (random() & (fgMode === 0x3000000 ? 0xffffff : 0xff)) | ((random() % 2) * 0x4000000);
    const bg = bgMode | (random() & (bgMode === 0x3000000 ? 0xffffff : 0xff)) | ((random() % 2) * 0x8000000);
    return [fg,bg];
  });
  const style = ['block','bar','underline','outline','none'][random() % 5];
  run(`seeded-${i}`, cols, rows, cell, [cols * cell[0] + 0.125, rows * cell[1] + 0.25], cells,
    {x: random() % cols, y: random() % rows, width: random() % 2 + 1, style, cursorWidth: (random() % 8 + 1) / 2, dpr: 1.5});
}
const output = path.resolve('crates/xterm-web/tests/fixtures/webgl-rectangles.json');
await writeFile(output, JSON.stringify({seed: '0x57e61a23', inputs, colors, palette, cases}) + '\n');
console.log(`${cases.length} source RectangleRenderer cases -> ${output}`);
