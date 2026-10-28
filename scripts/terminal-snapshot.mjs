// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// Lossless state projection shared by whole-terminal and handler-family oracles.
export const hex = s => Array.from({length:s.length},(_,i)=>s.charCodeAt(i).toString(16)).join(',');
function buffer(b) {
  const rows=[];
  for(let y=0;y<b.lines.length;y++) {
    const l=b.lines.get(y), cells=[];
    for(let x=0;x<l.length;x++) {
      const c=l.getCell(x),e=c.extended;
      cells.push([c.content>>>0,c.fg>>>0,c.bg>>>0,hex(c.getChars()),e.ext>>>0,e.urlId].join(','));
    }
    rows.push(`${+l.isWrapped}:${cells.join(';')}`);
  }
  return [b.x,b.y,b.ybase,b.ydisp,b.scrollTop,b.scrollBottom].join(',')+'/'+rows.join('/');
}
export function snapshot(t) {
  const c=t._core, a=c._inputHandler.getAttrData(), m=c.coreService.decPrivateModes;
  return [c.cols,c.rows,+(c.buffers.active===c.buffers.alt),a.fg>>>0,a.bg>>>0,a.extended.ext>>>0,a.extended.urlId,
    +c.coreService.modes.insertMode,+m.origin,+m.wraparound,+m.reverseWraparound,+m.applicationCursorKeys,
    +m.applicationKeypad,+m.bracketedPasteMode,+m.sendFocus,+m.synchronizedOutput].join(',')+
    '|'+buffer(c.buffers.normal)+'|'+buffer(c.buffers.alt);
}
