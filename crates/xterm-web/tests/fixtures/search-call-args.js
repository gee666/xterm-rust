// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
// The same assertions run against the unchanged source and the Rust bindings.
async function checkSearchCallArgs(term, addon, write, equal) {
  let cases = 0;
  const check = (label, actual, expected) => {
    try { equal(actual, expected); } catch (error) { throw Error(label + ': ' + error.message); }
  };
  await write('a a\r\nrow1\r\nrow2\r\nrow3\r\nrow4');
  const values = [undefined, null, false, 0, -0, NaN, '', 0n, true, 1, 'yes', {}, [], Symbol('flag'), 1n];
  const decorations = {matchOverviewRuler: '#ff0000', activeMatchColorOverviewRuler: '#00ff00'};
  for (const withDecorations of [false, true]) {
    const options = withDecorations ? {decorations} : undefined;
    for (const value of values) {
      addon.findNext('');
      check('initial match', addon.findNext('a', options), true);
      let event;
      const listener = addon.onDidChangeResults(result => event = result);
      addon.clearDecorations(value);
      check('retained next result', addon.findNext('a', options), true);
      check('retained cached term', term.getSelectionPosition().start.x, value ? 2 : 0);
      if (withDecorations) {
        check('retained highlight cache', event, value
          ? {resultIndex: -1, resultCount: 0} : {resultIndex: 0, resultCount: 2});
      }
      listener.dispose();
      cases++;
    }
  }
  for (const direction of ['findNext', 'findPrevious']) {
    for (const value of values) {
      addon.findNext('');
      term.scrollToBottom();
      check('pre-search viewport', term.buffer.active.viewportY, 2);
      let reads = 0;
      const internal = {get noScroll() {
        reads++;
        check('getter before selection', term.hasSelection(), false);
        check('getter before scrolling', term.buffer.active.viewportY, 2);
        return value;
      }};
      check('match with internal option', addon[direction]('a', undefined, internal), true);
      check('single late getter read', reads, 1);
      check('getter controls scrolling', term.buffer.active.viewportY, value ? 2 : 0);
      check('selected row', term.getSelectionPosition().start.y, 0);
      cases++;
    }
    let reads = 0;
    const internal = {get noScroll() { reads++; return true; }};
    check('empty term', addon[direction]('', undefined, internal), false);
    check('empty skips internal option', reads, 0);
    check('no match', addon[direction]('missing', undefined, internal), false);
    check('no match still reads internal option', reads, 1);
    cases += 2;
    reads = 0;
    let syntaxError = false;
    try { addon[direction]('[', {regex: true}, internal); }
    catch (error) { if (!(error instanceof SyntaxError)) throw error; syntaxError = true; }
    check('invalid regex throws', syntaxError, true);
    check('engine error precedes internal getter', reads, 0);
    cases++;
    addon.findNext('a');
    term.scrollToBottom();
    const sentinel = {internalSearchFailure: true};
    let after = 0;
    const afterListener = addon.onAfterSearch(() => after++);
    let caught = false;
    try { addon[direction]('a', undefined, {get noScroll() { throw sentinel; }}); }
    catch (error) { if (error !== sentinel) throw Error('internal getter exception identity changed'); caught = true; }
    check('getter exception propagated', caught, true);
    check('engine clears old selection before getter exception', term.hasSelection(), false);
    check('getter exception prevents scrolling', term.buffer.active.viewportY, 2);
    check('getter exception prevents after-search event', after, 0);
    afterListener.dispose();
    cases++;
    reads = 0;
    const beforeListener = addon.onBeforeSearch(() => { throw sentinel; });
    caught = false;
    try { addon[direction]('a', undefined, internal); }
    catch (error) { if (error !== sentinel) throw Error('before-search exception identity changed'); caught = true; }
    beforeListener.dispose();
    check('before-search exception propagated', caught, true);
    check('before-search exception precedes internal getter', reads, 0);
    cases++;
    addon.findNext('');
    reads = 0;
    check('nested search from internal getter', addon[direction]('a', undefined, {get noScroll() {
      reads++;
      check('nested match', addon.findNext('a', undefined, {noScroll: true}), true);
      return true;
    }}), true);
    check('nested getter called once', reads, 1);
    check('outer selection survives nested call', term.getSelection(), 'a');
    cases++;
  }
  let coerced = false;
  const truthy = {valueOf() { coerced = true; throw Error('unexpected valueOf'); },
    toString() { coerced = true; throw Error('unexpected toString'); }};
  addon.clearDecorations(truthy);
  term.clearSelection();
  term.scrollToBottom();
  check('inherited getter option', addon.findNext('a', undefined,
    Object.create({get noScroll() { return truthy; }})), true);
  check('inherited noScroll truthiness', term.buffer.active.viewportY, 2);
  check('boolean options never coerce objects', coerced, false);
  cases++;
  for (const value of [undefined, null, false, 0, '', Symbol('internal'), 0n]) {
    addon.findNext('');
    term.scrollToBottom();
    check('boxed primitive internal options', addon.findNext('a', undefined, value), true);
    check('absent noScroll scrolls', term.buffer.active.viewportY, 0);
    cases++;
  }
  const unloaded = new addon.constructor();
  let unloadedReads = 0;
  let unloadedFailure = false;
  try { unloaded.findNext('a', undefined, {get noScroll() { unloadedReads++; }}); }
  catch (error) { check('unloaded message', error.message, 'Cannot use addon until it has been loaded'); unloadedFailure = true; }
  finally { unloaded.dispose(); }
  check('unloaded search fails', unloadedFailure, true);
  check('unloaded search does not read internal option', unloadedReads, 0);
  cases++;
  return cases;
}
