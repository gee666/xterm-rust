// Test adapter only. The actual output is encoded by Rust through the callback.
function withPort(factory, run) {
  const owned = [];
  class Port {
    constructor() {
      this.native = factory();
      owned.push(this.native);
      this._codeToVk = this.native.table(0);
      this._codeToScancode = this.native.table(1);
      this._keyToControlChar = this.native.table(2);
    }
    evaluateKeyboardEvent(event, down) {
      return { type: 0, cancel: true, key: this.native.encode(event, down) };
    }
  }
  try { return run(Port); }
  finally { for (const native of owned) native.free(); }
}

export function compareWin32Prototypes(factory) {
  const failures = [];
  for (const spec of prototypeCases) {
    const expected = capturePrototypeCase(Win32InputMode, spec);
    const actual = withPort(factory, Port => capturePrototypeCase(Port, spec));
    if (JSON.stringify(actual) !== JSON.stringify(expected)) failures.push({spec, expected, actual});
  }
  const independent = withPort(factory, Port => compareIndependentWin32(Win32InputMode, Port));
  console.log(`Win32 comparison: ${prototypeCases.length} retained and ${independent.cases} independent cases`);
  return { cases: prototypeCases.length, failures, independent };
}
