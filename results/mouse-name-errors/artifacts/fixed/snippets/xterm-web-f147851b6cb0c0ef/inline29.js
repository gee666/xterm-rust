
export function imageAddonOptions(options) {
  return Object.assign({}, {
    enableSizeReports: true, pixelLimit: 16777216, sixelSupport: true,
    sixelScrolling: true, sixelPaletteLimit: 4096, sixelSizeLimit: 33554432,
    storageLimit: 128, showPlaceholder: true, iipSupport: true,
    iipSizeLimit: 33554432, kittySupport: true, kittySizeLimit: 33554432
  }, options);
}
export function imageOptionNumber(value) { return +value; }
// Preserve the operand expression too: some engines include it in TypeErrors.
function kittyEncodedLimit() {
  return Math.ceil(this._opts.kittySizeLimit * 4 / 3);
}
export function imageKittyEncodedLimit(kittySizeLimit) {
  return kittyEncodedLimit.call({_opts: {kittySizeLimit}});
}
// Keep this operand spelling for engine-specific generated TypeError messages.
function iipEncodedLimit() {
  return Math.ceil(this._opts.iipSizeLimit * 4 / 3);
}
export function imageIipEncodedLimit(iipSizeLimit) {
  return iipEncodedLimit.call({_opts: {iipSizeLimit}});
}
