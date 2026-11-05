// Test-only adapter: the callback reaches real Rust with production LiveKeyEvent.
export function kittySourceExpected(event) {
  return new KittyKeyboard().evaluate(event, 8, 1, false).key;
}
export function installKittyDomProbe(callback) {
  const name = '__kitty_dom_probe__';
  const original = Object.getOwnPropertyDescriptor(Object.prototype, name);
  Object.defineProperty(Object.prototype, name, {
    configurable: true, get() { return callback(this); }
  });
  return () => {
    if (original) Object.defineProperty(Object.prototype, name, original);
    else delete Object.prototype[name];
  };
}
export function kittyThrowValue(marker) { return { valueOf() { throw marker; } }; }
export function compareKittyRealm(factory) {
  const owned = [];
  class Port {
    constructor() {
      this.native = factory();
      owned.push(this.native);
      ['_functionalKeyCodes', '_csiTildeKeys', '_csiLetterKeys', '_ss3FunctionKeys']
        .forEach((field, index) => { this[field] = this.native.table(index); });
    }
    evaluate(event, flags, type, mac) { return this.native.encode(event, flags, type, mac); }
  }
  try {
    const result = compareKittyLookups(KittyKeyboard, Port);
    console.log('Kitty realm comparisons: ' + result.cases + ', failures: ' + result.failures.length);
    return result;
  } finally {
    for (const native of owned) native.free();
  }
}
