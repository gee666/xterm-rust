// Snapshot fields are observable through Object.keys, spread and JSON. Keep
// them live rather than adding a serializer-specific toJSON approximation.
const define = Object.defineProperty;
const descriptor = Object.getOwnPropertyDescriptor;
const prototype = Object.getPrototypeOf;
function expose(value, fields) {
  define(value, '__wbg_ptr', { enumerable: false });
  const proto = prototype(value);
  for (const field of fields) {
    define(value, field, { ...descriptor(proto, field), configurable: true, enumerable: true });
  }
  return value;
}
export function shapeBufferLine(value) {
  return expose(value, ['_line']);
}
export function shapeBufferCell(value) {
  return expose(value, ['fg', 'bg', 'extended', 'content', 'combinedData']);
}
export function shapeExtendedCell(value) {
  expose(value, ['_ext', '_urlId']);
  if (value.payload !== undefined) {
    define(value, 'payload', { ...descriptor(prototype(value), 'payload'), configurable: true, enumerable: true });
  }
  return value;
}
