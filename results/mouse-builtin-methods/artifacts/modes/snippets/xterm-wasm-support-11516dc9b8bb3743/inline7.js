
export function unicodeVersion(provider) { return String(provider.version); }
export function unicodeWidth(provider, codepoint) { return provider.wcwidth(codepoint); }
export function unicodeProperties(provider, codepoint, preceding) {
  return provider.charProperties(codepoint, preceding) >>> 0;
}
export function unknownUnicodeVersion(version) {
  return new Error('unknown Unicode version "' + version + '"');
}
