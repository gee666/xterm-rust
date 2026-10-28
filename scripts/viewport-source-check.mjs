// Copyright (c) 2024 The xterm.js authors. All rights reserved. MIT license.
// Scope: unchanged Viewport methods; the real DOM constructor is tested separately.
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
const require=createRequire(import.meta.url);
const esbuild=require('../source_repo/node_modules/esbuild');
const built=esbuild.buildSync({entryPoints:['source_repo/src/browser/Viewport.ts'],bundle:true,write:false,format:'esm',platform:'node',tsconfig:'source_repo/src/browser/tsconfig.json'});
const {Viewport}=await import(`data:text/javascript;base64,${Buffer.from(built.outputFiles[0].contents).toString('base64')}`);
function fixture(){
 const state={row:2,cell:10,canvas:40,count:20,sync:false,top:12,writes:[],dims:[],requests:[],callbacks:[],hook:()=>{},fail:undefined,immediate:false};
 const call=name=>{state.hook(name);if(state.fail===name)throw sentinel;};
 const sentinel={kind:'host failure'};
 const view=Object.create(Viewport.prototype);
 Object.assign(view,{_queuedAnimationFrame:undefined,_latestYDisp:undefined,_isSyncing:false,_isHandlingScroll:false,_suppressOnScrollHandler:false,_needsSyncOnRender:false});
 view._bufferService={buffer:{get ydisp(){call('row');return state.row;},lines:{get length(){call('count');return state.count;}}}};
 view._coreService={decPrivateModes:{get synchronizedOutput(){call('mode');return state.sync;}}};
 view._renderService={dimensions:{css:{cell:{get height(){call('cell');return state.cell;}},canvas:{get height(){call('canvas');return state.canvas;}}}},
  addRefreshCallback(cb){call('queue');const id=state.callbacks.length;if(state.immediate)cb();else state.callbacks.push(cb);return id;}};
 view._scrollableElement={getScrollPosition(){call('top');return{scrollTop:state.top};},setScrollPosition(value){state.top=value.scrollTop;state.writes.push(value);call('position');},setScrollDimensions(value){state.dims.push(value);call('dimensions');}};
 view._onRequestScrollLines={fire(diff){state.requests.push(diff);call('request');}};
 const flush=()=>{const callbacks=state.callbacks.splice(0);for(const callback of callbacks)callback();};
 return{view,state,sentinel,flush};
}
let cases=0;
{
 const{view,state,flush}=fixture();view.queueSync();view.queueSync();assert.equal(state.callbacks.length,1);state.cell=12.5;flush();
 assert.deepEqual(state.dims,[{height:40,scrollHeight:250}]);assert.deepEqual(state.writes,[{scrollTop:25}]);assert.equal(view._queuedAnimationFrame,undefined);cases++;
}
{
 const{view,state,flush}=fixture();view.queueSync(4);view.queueSync(9);flush();assert.equal(view._latestYDisp,9);assert.deepEqual(state.writes,[]);
 view._latestYDisp=undefined;state.row=5;view.queueSync();flush();assert.deepEqual(state.writes,[{scrollTop:50}]);cases++;
}
{
 const{view,state}=fixture();state.immediate=true;view.queueSync();assert.equal(view._queuedAnimationFrame,0);view.queueSync(3);assert.equal(view._latestYDisp,3);cases++;
}
{
 const{view,state}=fixture();state.sync=true;view._sync();assert.equal(view._needsSyncOnRender,true);assert.deepEqual(state.dims,[]);
 view._needsSyncOnRender=false;state.sync=false;view._sync();assert.equal(state.dims.length,1);cases++;
}
{
 const{view,state}=fixture();state.hook=name=>{if(name==='dimensions')view._handleScroll({scrollTop:99});if(name==='canvas')view._sync();};view._sync();assert.deepEqual(state.requests,[]);cases++;
}
{
 const{view,state}=fixture();view.scrollLines(2.5);view.scrollToLine(4.5);view.scrollToLine(6.5,true);view.handleTouchScroll(7.5);
 assert.deepEqual(state.writes,[{reuseAnimation:true,scrollTop:37},{reuseAnimation:true,scrollTop:45},{reuseAnimation:false,scrollTop:65},{scrollTop:57.5}]);assert.equal(view._latestYDisp,6.5);cases++;
}
{
 const{view,state}=fixture();state.hook=name=>{if(name==='request'){assert.equal(view._latestYDisp,3);view._handleScroll({scrollTop:100});}};view._handleScroll({scrollTop:29});assert.deepEqual(state.requests,[1]);cases++;
}
{
 const{view,state}=fixture();state.row=1;view._handleScroll({scrollTop:-5});assert(Object.is(view._latestYDisp,-0));view._handleScroll({scrollTop:NaN});assert(Number.isNaN(view._latestYDisp));view._handleScroll({scrollTop:Infinity});assert.equal(view._latestYDisp,Infinity);cases++;
}
{
 const{view,state,sentinel}=fixture();state.fail='dimensions';assert.throws(()=>view._sync(),value=>value===sentinel);state.fail=undefined;assert.equal(view._isSyncing,true);assert.equal(view._suppressOnScrollHandler,true);view._handleScroll({scrollTop:100});assert.deepEqual(state.requests,[]);cases++;
}
{
 const{view,state,sentinel}=fixture();state.fail='request';assert.throws(()=>view._handleScroll({scrollTop:30}),value=>value===sentinel);assert.equal(view._isHandlingScroll,true);state.fail=undefined;view._handleScroll({scrollTop:100});assert.deepEqual(state.requests,[1]);cases++;
}
assert.equal(cases,10);console.log(`Viewport source method checks: ${cases} passed. Constructor/DOM integration not covered here.`);
