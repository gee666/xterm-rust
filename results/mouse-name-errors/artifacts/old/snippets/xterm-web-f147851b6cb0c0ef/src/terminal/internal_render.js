// Shape/ownership adapter only. Every scalar comes from the Rust renderer.
export function dimensionsView(read) {
  const size = (group, kind, keys = ['width', 'height']) => {
    const value = {};
    for (const key of keys) {
      Object.defineProperty(value, key, {
        enumerable: true,
        get: () => read()[group][kind][key]
      });
    }
    return Object.freeze(value);
  };
  return Object.freeze({
    css: Object.freeze({ canvas: size('css', 'canvas'), cell: size('css', 'cell') }),
    device: Object.freeze({
      canvas: size('device', 'canvas'), cell: size('device', 'cell'),
      char: size('device', 'char', ['width', 'height', 'left', 'top'])
    })
  });
}
