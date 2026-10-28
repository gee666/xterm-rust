// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.

/** Canonicalize record delimiters for TSV with encoded, single-line payloads.
 * Do not normalize standalone CR, escaped payload text, BOMs or token content.
 * Callers retain and hash the original bytes separately from this comparison.
 */
export function canonicalTsvRecords(text) {
  return text.replace(/\r\n/g, '\n');
}
