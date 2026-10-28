// Copyright (c) 2019-2026 The xterm.js authors. All rights reserved. MIT license.
import fs from 'node:fs';
import path from 'node:path';
import { sourceLoader } from './source-ts-loader.mjs';
const {source,load,hashes,typescript}=sourceLoader();
const {Linkifier}=load(path.join(source,'src/browser/Linkifier.ts'));
let seed=0x11f1e255;const initialSeed=seed;
const next=()=>(seed=(Math.imul(seed,1664525)+1013904223)>>>0);
const records=[];
const encode=links=>links===undefined?'-':links.map(({link})=>[link.text,link.range.start.x,link.range.start.y,link.range.end.x,link.range.end.y].join(':')).join(',');
for(let scenario=0;scenario<2048;scenario++){
  const count=1+next()%5;const cols=1+next()%80;const position={x:1+next()%cols,y:1+next()%4};
  const target=Object.create(Linkifier.prototype);target._bufferService={cols};target._linkProviderService={linkProviders:Array(count)};target._activeProviderReplies=new Map();
  let selected=[];target._handleNewLink=link=>selected.push(link.link.text);
  const pending=Array.from({length:count},(_,i)=>i);let provided=false;
  const cache=()=>[...target._activeProviderReplies].sort((a,b)=>a[0]-b[0]).map(([i,links])=>`${i}=${links===undefined?'-':links.map(({link})=>link.text).join(',')}`).join(';');
  while(pending.length){
    const index=pending.splice(next()%pending.length,1)[0];
    const kind=next()%4;
    const links=kind===0?undefined:Array.from({length:kind===1?0:1+next()%4},(_,j)=>{
      const start={x:1+next()%cols,y:1+next()%4};
      const end={x:1+next()%cols,y:start.y+next()%3};
      return{link:{text:`s${scenario}p${index}l${j}`,range:{start,end}}};
    });
    const input=encode(links);
    target._activeProviderReplies.set(index,links);selected=[];
    provided=target._checkLinkProviderResult(index,position,provided);
    if(target._activeProviderReplies.size===count)target._removeIntersectingLinks(position.y,target._activeProviderReplies);
    records.push([scenario,'reply',count,cols,position.x,position.y,index,input,selected.join(','),cache()].join('\t'));
  }
  for(let probe=0;probe<3;probe++){
    const position={x:1+next()%cols,y:1+next()%5};selected=[];let provided=false;
    for(let index=0;index<count;index++){
      if(target._activeProviderReplies.get(index))provided=target._checkLinkProviderResult(index,position,provided);
    }
    records.push([scenario,'cache',count,cols,position.x,position.y,'-','-',selected.join(','),cache()].join('\t'));
  }
}
fs.mkdirSync('crates/xterm-web/tests/fixtures',{recursive:true});fs.mkdirSync('results/links',{recursive:true});
fs.writeFileSync('crates/xterm-web/tests/fixtures/link-replies.tsv',records.join('\n')+'\n');
const result={command:'node scripts/link-replies-oracle.mjs',node:process.version,typescript,seed:initialSeed,scenarios:2048,operations:records.length,
  scope:'original Linkifier reply-priority, cache and overlap methods, no DOM substitutions claimed',sources:Object.fromEntries(hashes)};
fs.writeFileSync('results/links/reply-oracle.json',JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result,null,2));
