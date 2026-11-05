// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Appended to unchanged CharAtlasCache.ts. Real TextureAtlas and canvas work,
// with source exports and private cache state used only for test observation.
globalThis.sourceCacheScenario = name => {
  const trace = [], atlases = [], canvases = [];
  const originalCreate = document.createElement;
  let removals = 0, failRemove = false, failCreate = false, reenter, createReentry;
  document.createElement = function(...args) {
    if (failCreate) { failCreate = false; trace.push('create-failure'); throw globalThis.cacheFailure; }
    if (createReentry) { const f = createReentry; createReentry = undefined; f(); }
    const canvas = originalCreate.apply(this, args);
    if (args[0] === 'canvas') {
      canvases.push(canvas);
      const remove = canvas.remove;
      canvas.remove = function() {
        removals++;
        trace.push('remove');
        if (reenter) { const callback = reenter; reenter = undefined; callback(); }
        if (failRemove) { failRemove = false; throw globalThis.cacheFailure; }
        return remove.call(this);
      };
    }
    return canvas;
  };
  const color = (css, rgba) => ({ css, rgba });
  const white = color('#eeeeee', 0xeeeeeeff), bg = color('#102030', 0x102030ff);
  const contrastCache = { clear() {}, getColor() {}, setColor() {} };
  const colors = { foreground:white, background:bg, ansi:Array(256).fill(white), contrastCache, halfContrastCache:contrastCache };
  const options = { letterSpacing:0, lineHeight:1, fontFamily:'monospace', fontSize:15,
    fontWeight:'normal', fontWeightBold:'bold', allowTransparency:false,
    drawBoldTextInBrightColors:true, minimumContrastRatio:1 };
  const unicode = { getStringCellWidth:s=>s.length };
  const log = { trace(){},debug(){},info(){},warn(){},error(){} };
  let failServices = false;
  const owner = label => Object.defineProperty({}, '_core', { get() {
    trace.push('services:'+label);
    if (failServices) { failServices = false; throw globalThis.cacheFailure; }
    return { unicodeService:unicode, _logService:log };
  }});
  const a = owner('a'), b = owner('b'), c = owner('c');
  const ids = [], versions = [];
  let baseVersion;
  function acquire(t, size=15) {
    const atlas = acquireTextureAtlas(t, {...options,fontSize:size}, colors, 10,20,8,16,1,4096);
    if (!atlases.includes(atlas)) atlases.push(atlas);
    ids.push(atlases.indexOf(atlas));
    baseVersion ??= atlas.pages[0].version;
    versions.push(atlas.pages[0].version - baseVersion);
    return atlas;
  }
  const states = [], pixels = [];
  function snapshot(atlas) {
    let hash = 2166136261;
    const page = atlas.pages[0];
    for (const byte of page.canvas.getContext('2d').getImageData(0,0,512,512).data) hash = Math.imul(hash ^ byte,16777619) >>> 0;
    pixels.push(hash);
    states.push([atlas.pages.length, atlas.pageLayoutVersion]);
  }
  try {
    globalThis.cacheFailure = {};
    if (name === 'sharing') {
      const first = acquire(a); const shared = acquire(b); acquire(a);
      first.getRasterizedGlyph(65,0,0,0,false,undefined); snapshot(shared);
      shared.clearTexture(); snapshot(first);
      removeTerminalFromCache(a); snapshot(shared);
      removeTerminalFromCache(b); removeTerminalFromCache(b); acquire(c);
    } else if (name === 'move') {
      acquire(a); acquire(b); acquire(a,16); removeTerminalFromCache(b);
      acquire(c,16); removeTerminalFromCache(a); removeTerminalFromCache(c);
    } else if (name === 'failure') {
      acquire(a); failRemove = true;
      try { acquire(a,16); } catch(e) { trace.push(e === globalThis.cacheFailure ? 'caught' : 'wrong error'); }
      acquire(a); acquire(a,16); removeTerminalFromCache(a);
    } else if (name === 'creation_failure') {
      acquire(a); failCreate = true;
      try { acquire(a,16); } catch(e) { trace.push(e === globalThis.cacheFailure ? 'caught' : 'wrong error'); }
      acquire(a); removeTerminalFromCache(a);
    } else if (name === 'services_failure') {
      acquire(a); failServices = true;
      try { acquire(a,16); } catch(e) { trace.push(e === globalThis.cacheFailure ? 'caught' : 'wrong error'); }
      acquire(a); removeTerminalFromCache(a);
    } else if (name === 'reentry') {
      acquire(a); acquire(b,16);
      reenter = () => removeTerminalFromCache(a);
      removeTerminalFromCache(a); acquire(b,16); removeTerminalFromCache(b);
    } else if (name === 'canvas_reentry') {
      createReentry = () => acquire(b);
      acquire(a); acquire(c); removeTerminalFromCache(a); removeTerminalFromCache(b); removeTerminalFromCache(c);
    } else if (name === 'create_reentry') {
      const d = Object.defineProperty({}, '_core', { get() {
        trace.push('services:d'); acquire(b); return {unicodeService:unicode,_logService:log};
      }});
      acquire(d); acquire(a); removeTerminalFromCache(d); removeTerminalFromCache(a); removeTerminalFromCache(b);
    } else throw new Error('Unknown scenario '+name);
    return JSON.stringify({ids,versions,trace,states,pixels,removals});
  } finally {
    document.createElement = originalCreate;
    reenter = undefined; createReentry = undefined; failRemove = false;
    // Each scenario is isolated after observing its source partial state.
    for (const entry of charAtlasCache) entry.atlas.dispose();
    charAtlasCache.length = 0;
    for (const canvas of canvases) canvas.remove();
  }
};
