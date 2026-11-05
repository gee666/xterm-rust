
export function imageAddonOptions(options) {
  return Object.assign({}, {
    enableSizeReports: true, pixelLimit: 16777216, sixelSupport: true,
    sixelScrolling: true, sixelPaletteLimit: 4096, sixelSizeLimit: 33554432,
    storageLimit: 128, showPlaceholder: true, iipSupport: true,
    iipSizeLimit: 33554432, kittySupport: true, kittySizeLimit: 33554432
  }, options);
}
export function imageOptionNumber(value) { return +value; }
