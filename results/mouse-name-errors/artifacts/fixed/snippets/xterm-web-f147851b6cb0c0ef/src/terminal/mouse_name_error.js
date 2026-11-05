// Error text crosses this boundary as a JS string to retain lone UTF16 surrogates.
// Mode lookup, validation, state changes and event dispatch remain in Rust.
export function mouseNameError(kind, name) {
  return new Error(`unknown ${kind} "${name}"`);
}
