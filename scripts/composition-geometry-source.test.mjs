// Licensed under MIT. Execute the unchanged geometry body, not Rust logic.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { boundsCases, geometryModel } from '../crates/xterm-web/tests/fixtures/composition-geometry-cases.js';

const body = await readFile(new URL('../crates/xterm-web/tests/fixtures/composition-geometry-body.js', import.meta.url), 'utf8');
for (const [name, width, height] of boundsCases) {
  test('source IME bounds ' + name, () => {
    const area = { style: {} };
    const overlay = { style: {}, getBoundingClientRect: () => ({ width, height }) };
    const model = geometryModel(area, overlay, body);
    model.updateCompositionElements(true);
    const expected = {
      finite: ['6.5px', '4.5px', '4.5px'],
      zero: ['1px', '1px', '0px'],
      negative: ['1px', '1px', '-3px'],
      'NaN-width': ['NaNpx', '4.5px', '4.5px'],
      'NaN-height': ['6.5px', 'NaNpx', 'NaNpx'],
      'NaN-both': ['NaNpx', 'NaNpx', 'NaNpx'],
      infinite: ['Infinitypx', 'Infinitypx', 'Infinitypx'],
    }[name];
    assert.deepEqual(['width', 'height', 'lineHeight'].map(key => area.style[key]), expected);
    assert.equal(overlay.style.maxWidth, '9px');
  });
}
