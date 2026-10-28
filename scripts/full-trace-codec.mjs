// Exact UTF-16 transport; neither scalar normalization nor prefix parsing.
export function decodeUnits(text) {
  if (text === '') return [];
  if (typeof text !== 'string' || !/^[0-9a-f]{1,4}(,[0-9a-f]{1,4})*$/i.test(text)) {
    throw new Error('Invalid UTF-16 hex encoding');
  }
  return text.split(',').map(unit => Number.parseInt(unit, 16));
}
export const fromUnits = units => units.map(unit => String.fromCharCode(unit)).join('');
