// Browser language/DOM operations only. Matching loops and font shaping are Rust.
export function sortedFallbacks(options, defaults) {
  return (options?.fallbackLigatures ?? defaults).sort((a, b) => b.length - a.length);
}
export function fontFeatures(options) { return options?.fontFeatureSettings; }
export function terminalElement(terminal) { return terminal.element; }
export function registerJoiner(terminal, callback) { return terminal.registerCharacterJoiner(callback); }
export function deregisterJoiner(terminal, id) { return terminal?.deregisterCharacterJoiner(id); }
export function setFontFeatures(terminal, value) { terminal.element.style.fontFeatureSettings = value ?? '"calt" on'; }
export function clearFontFeatures(terminal) { if (terminal?.element) terminal.element.style.fontFeatureSettings = ''; }
export function fontFamily(terminal) { return terminal.options.fontFamily; }
export function debugEnabled(terminal) { return terminal.options.logLevel === 'debug'; }
export function refreshFont(terminal) { terminal.refresh(0, terminal.rows - 1); }
export function fontFailure(previous) { console.debug(previous, new Error('Failure while loading font')); }
export function finishFontLoad(promise, loaded, failed) { promise.then(loaded).catch(failed); }
export function fallbackInBounds(patterns, index) { return index < patterns.length; }
export function fallbackPattern(patterns, index) { return patterns[index]; }
export function startsWith(text, pattern, index) { return text.startsWith(pattern, index); }
export function fallbackEnd(patterns, patternIndex, textIndex) { return textIndex + patterns[patternIndex].length; }
export function fallbackNext(patterns, patternIndex, textIndex) { return textIndex + (patterns[patternIndex].length - 1) + 1; }
