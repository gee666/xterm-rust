// Copyright (c) 2020, 2023 The xterm.js authors. MIT license.
// SIXEL's original decoder throws engine-generated WASM diagnostics. Their text
// differs across browsers. Reify only the trap, not any terminal/image behavior.
export function sixelTrapText(kind) {
  const section = (id, payload) => [id, payload.length, ...payload];
  let types = [1, 0x60, 2, 0x7f, 0x7f, 1, 0x7f];
  let functions = [1, 0], resources = [], elements = [], bodies;
  switch (kind) {
    case 'remainder':
      bodies = [[0, 0x41, 1, 0x41, 0, 0x70, 0x0b]];
      break;
    case 'memory':
      resources = section(5, [1, 0, 1]);
      bodies = [[0, 0x41, 0x80, 0x80, 4, 0x28, 2, 0, 0x0b]];
      break;
    case 'signature':
      types = [2, ...types.slice(1), 0x60, 0, 0];
      functions = [2, 0, 1];
      resources = section(4, [1, 0x70, 0, 1]);
      elements = section(9, [1, 0, 0x41, 0, 0x0b, 1, 1]);
      bodies = [[0, 0x41, 0, 0x41, 0, 0x41, 0, 0x11, 0, 0, 0x0b], [0, 0x0b]];
      break;
    default:
      throw new TypeError('Unknown SIXEL trap kind');
  }
  const bytes = new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0,
    ...section(1, types), ...section(3, functions), ...resources,
    ...section(7, [1, 6, 100, 101, 99, 111, 100, 101, 0, 0]), ...elements,
    ...section(10, [bodies.length, ...bodies.flatMap(body => [body.length, ...body])])]);
  const module = new WebAssembly.Module(bytes);
  const probe = {
    _wasm: new WebAssembly.Instance(module).exports,
    decode(length) {
      // Capture here so WebKit attributes the trap to the WASM call, not the
      // outer diagnostic helper invocation after JIT inlining.
      try {
        this._wasm.decode(0, length);
      } catch (error) {
        return String(error);
      }
      throw new Error('SIXEL diagnostic module did not trap');
    }
  };
  return probe.decode(0);
}
