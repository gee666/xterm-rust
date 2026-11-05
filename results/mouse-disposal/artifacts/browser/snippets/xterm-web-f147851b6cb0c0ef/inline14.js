
export function fastCacheNew() { return {width:'',height:'',top:'',left:'',bottom:'',right:'',position:'',contain:'none',className:''}; }
export function fastCacheSet(cache,key,value) { 'use strict'; cache[key]=value; }
export function fastCachedStyle(node,key,cache) { 'use strict'; node.style[key]=cache[key]; }
export function fastCachedClass(node,cache) { 'use strict'; node.className=cache.className; }
export function fastLayerHint(node,enabled) { 'use strict'; node.style.transform=enabled?'translate3d(0px, 0px, 0px)':''; }
export function fastToggle(node,name,force) { 'use strict'; node.classList.toggle(name,force); return node.className; }
export function fastAttribute(node,name,value) { 'use strict'; node.setAttribute(name,value); }
export function fastBeginStyle(node) { return node.style; }
export function fastAssignStyle(style,key,value) { 'use strict'; style[key]=value; }
