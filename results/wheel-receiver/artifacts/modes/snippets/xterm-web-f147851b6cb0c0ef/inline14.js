
export function pairEvent(ak, av, bk, bv) { return { [ak]: av, [bk]: bv }; }
export function keyEvent(key, domEvent) { return { key, domEvent }; }
export function dimensionsEvent(cw, ch, sw, sh, dw, dh, tw, th, w, h, left, top) {
  return {
    css: { canvas: { width: cw, height: ch }, cell: { width: sw, height: sh } },
    device: { canvas: { width: dw, height: dh }, cell: { width: tw, height: th },
      char: { width: w, height: h, left, top } }
  };
}
