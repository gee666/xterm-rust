
const FALLBACK_PIXEL_LIMIT = 2500000;
const PIXELS_PER_MEGABYTE = 250000;
export function imageStoragePixels(value) {
  if (value < 0.5 || value > 1000) {
    throw RangeError('invalid storageLimit, should be at least 0.5 MB and not exceed 1G');
  }
  return (value / 4 * 1000000) >>> 0;
}
export function imageStorageInitialPixels(value) {
  try { return imageStoragePixels(value); }
  catch (error) {
    if (error instanceof Error) console.error(error.message);
    console.warn(`storageLimit is set to ${FALLBACK_PIXEL_LIMIT / PIXELS_PER_MEGABYTE} MB`);
    return FALLBACK_PIXEL_LIMIT;
  }
}
