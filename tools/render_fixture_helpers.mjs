// Shared read-only snapshots for browser renderer fixture capture.
export function captureCells(line, CellData) {
  return Array.from({ length: line.length }, (_, index) => {
    const cell = line.loadCell(index, new CellData());
    return { content: cell.content >>> 0, fg: cell.fg >>> 0, bg: cell.bg >>> 0,
      chars: Array.from({ length: cell.combinedData.length }, (_, i) => cell.combinedData.charCodeAt(i)),
      ext: cell.extended.ext >>> 0, url: cell.extended.urlId >>> 0 };
  });
}
export function utf16(text) {
  return Array.from({ length: text.length }, (_, index) => text.charCodeAt(index));
}
