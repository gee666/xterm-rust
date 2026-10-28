// Copyright (c) 2019, 2021 The xterm.js authors. MIT license.
// Shared operation trace, executed against unchanged source and the Rust view.
export function unicodeTrace(api) {
  const snapshots = [];
  const names = ['', '2', '01', '0', '4294967294', '4294967295',
    '__proto__', 'constructor', '\ud800', '\udc00', 'a\ud800b',
    '\ud800\udc00', '\u{f0000}', '\u{f0000}d800', '\ufffd'];
  const snapshot = () => snapshots.push({versions: api.versions, active: api.activeVersion});
  for (const name of names) {
    api.register({version: name, wcwidth: () => 1, charProperties: () => 2});
    snapshot();
  }
  for (const name of names) {
    api.activeVersion = name;
    snapshot();
  }
  // Replacing a key must not reorder it.
  api.register({version: '\ud800', wcwidth: () => 2, charProperties: () => 4});
  snapshot();
  const errors = [];
  for (const name of ['missing\ud800', '\u{f0000}0000']) {
    try { api.activeVersion = name; }
    catch (error) { errors.push({name: error.name, message: error.message}); }
    snapshot();
  }
  // Empty active version is the source sentinel, even after explicit selection.
  api.activeVersion = '';
  let versionReads = 0;
  try {
    api.register({
      get version() { return ++versionReads === 1 ? 'insert\ud800' : 'absent\udc00'; },
      wcwidth: () => 1, charProperties: () => 2
    });
  } catch (error) { errors.push({name: error.name, message: error.message}); }
  snapshot();
  return {snapshots, errors, versionReads};
}

export function registerThrowing(api, afterFirst) {
  const failure = {};
  let reads = 0;
  try {
    api.register({
      get version() { if (++reads > afterFirst) throw failure; return 'read\ud800'; },
      wcwidth: () => 1, charProperties: () => 2
    });
  } catch (error) { return {failure, error, reads}; }
  throw new Error('expected a throwing version getter');
}

// The same width operations run against unchanged source and the live WASM
// internal view. Public selection and the width service must share one registry.
export function unicodeWidthTrace(api, service) {
  const v6Emoji = service.getStringCellWidth('🤣🤣');
  const host = registerMutable(api);
  const widths = [service.getStringCellWidth('\ud800A\udc00'),
    service.getStringCellWidth('\ud800')];
  host.setWide(true);
  widths.push(service.getStringCellWidth('\ud800'));
  host.replace();
  widths.push(service.getStringCellWidth('A'));
  host.setFail(true);
  const errors = [];
  for (const text of ['\ud800', 'A']) {
    try { service.getStringCellWidth(text); errors.push(false); }
    catch (error) { errors.push(error === host.failure); }
  }
  host.setFail(false);
  widths.push(service.getStringCellWidth('AB'));
  const reentry = [];
  api.register({
    version: 'reentry', wcwidth: () => 2,
    charProperties(code, preceding) {
      reentry.push([code, preceding]);
      api.activeVersion = '6';
      return 4;
    }
  });
  api.activeVersion = 'reentry';
  const reentryWidth = service.getStringCellWidth('AB');
  return {v6Emoji, widths, errors, callbacks: host.trace, reentry, reentryWidth, active: api.activeVersion};
}

export function registerMutable(api) {
  const trace = [];
  const failure = {};
  let fail = false;
  let wide = false;
  const provider = {
    get version() { trace.push('version'); return '\ud800'; },
    wcwidth(code) { trace.push(['width', code, this === provider]); if (fail) throw failure; return wide ? 2 : 1; },
    charProperties(code, preceding) {
      trace.push(['properties', code, preceding, this === provider]);
      if (fail) throw failure;
      return wide ? 4 : 2;
    }
  };
  api.register(provider);
  api.activeVersion = '\ud800';
  return {
    trace, failure,
    setFail(value) { fail = value; },
    setWide(value) { wide = value; },
    replace() { api.register({version: '\ud800', wcwidth: () => 0, charProperties: () => 0}); }
  };
}
