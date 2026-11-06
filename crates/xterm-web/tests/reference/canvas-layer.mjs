// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
import {BaseRenderLayer} from 'webgl/renderLayer/BaseRenderLayer';
import {LinkRenderLayer} from 'webgl/renderLayer/LinkRenderLayer';
import {removeTerminalFromCache} from 'webgl/CharAtlasCache';
import {OptionsService} from 'common/services/OptionsService';
import {ThemeService} from 'browser/services/ThemeService';
import {UnicodeService} from 'common/services/UnicodeService';
import {UnicodeV6} from 'common/input/UnicodeV6';
import {LogService} from 'common/services/LogService';
import {Emitter} from 'common/Event';
import {CellData} from 'common/buffer/CellData';
import * as probe from '../canvas_layer_probe.js';
const dimensions=()=>({device:{cell:{width:10,height:20},char:{width:8,height:16,left:1,top:2},canvas:{width:80,height:80}},
 css:{cell:{width:10/1.5,height:20/1.5},canvas:{width:80/1.5,height:80/1.5}}});
const event=(x1=2,y1=1,x2=6,y2=1)=>({x1,y1,x2,y2,cols:8});
class Base extends BaseRenderLayer {
  reset(){probe.layerNote('reset');}
  handleGridChanged(terminal,start,end){probe.layerNote('grid:'+start+':'+end);}
}
export function sourceCanvasLayer(name) {
  probe.beginLayerCapture();
  let layer, terminal, theme, options, result;
  const container=document.createElement('div');document.body.appendChild(container);
  try {
    options=new OptionsService({fontFamily:'monospace',fontSize:14,cols:8,rows:4,theme:{foreground:'#eeeeee',background:'#102030'}});
    theme=new ThemeService(options);
    const unicode=new UnicodeService();unicode.register(new UnicodeV6());
    terminal={rows:4,cols:8,options:options.rawOptions,_core:{unicodeService:unicode,_logService:new LogService(options)}};
    const browser={mainDocument:document,get dpr(){return window.devicePixelRatio;}};
    const show=new Emitter(), hide=new Emitter();
    const make=(alpha=true)=>new Base(terminal,container,'test',3,alpha,browser,options,theme);
    if(name==='constructor_error') {
      probe.failLayer('context');try{make();}catch(error){probe.layerNote(error===probe.layerFailure()?'same':'different');}
      probe.failLayer('null');try{make();}catch(error){probe.layerNote(error.message);}
      probe.layerNote('children:'+container.children.length);
      return probe.endLayerCapture();
    }
    const base=name==='base'||name==='transparency_error'||name==='font_reentry';
    layer=base?make(false):new LinkRenderLayer(container,2,terminal,{onShowLinkUnderline:show.event,onHideLinkUnderline:hide.event},browser,options,theme);
    layer.resize(terminal,dimensions());probe.layerScheduled();
    const snapshot=()=>probe.layerSnapshot(layer._canvas);
    if(name==='link') {
      show.fire(event());snapshot();hide.fire(event(7,3,8,3));snapshot();
      show.fire(event(6,0,3,3));snapshot();layer.reset(terminal);layer.reset(terminal);snapshot();
    } else if(name==='colors') {
      show.fire({...event(),fg:257});snapshot();hide.fire(event());
      show.fire({...event(),fg:1});snapshot();
      try{show.fire({...event(),fg:0x1000001});}catch(error){probe.layerNote(error.name);}
      hide.fire(event());snapshot();
    } else if(name==='retained') {
      const e=event();show.fire(e);Object.assign(e,event(1,0,4,3));hide.fire(event());snapshot();
    } else if(name==='theme') {
      show.fire(event(6,0,3,3));
      options.options.theme={foreground:'#ff0000',background:'#112233'};
      snapshot();probe.layerNote('atlas:'+!!layer._charAtlas);probe.layerScheduled();
      layer._charAtlas.getRasterizedGlyph(65,0,0,0,false,undefined);
      probe.layerSnapshot(layer._charAtlas.pages[0].canvas);
      const sibling=new LinkRenderLayer(container,2,terminal,{onShowLinkUnderline:show.event,onHideLinkUnderline:hide.event},browser,options,theme);
      try{sibling.resize(terminal,dimensions());probe.layerNote('shared:'+(sibling._charAtlas===layer._charAtlas));probe.layerScheduled();}
      finally{sibling.dispose();}
    } else if(name==='dpr_reentry') {
      probe.reenterLayerDpr(()=>{options.options.theme={foreground:'#ff0000',background:'#112233'};});
      layer.resize(terminal,dimensions());
      layer._charAtlas.getRasterizedGlyph(65,0,0,0,false,undefined);
      probe.layerSnapshot(layer._charAtlas.pages[0].canvas);probe.layerScheduled();
    } else if(name==='resize') {
      show.fire(event());layer.resize(terminal,dimensions());layer.reset(terminal);snapshot();
      const dim=dimensions();dim.device.char.width=0;dim.device.char.height=0;
      const previous=layer._charAtlas;layer.resize(terminal,dim);
      probe.layerNote('retained:'+(previous===layer._charAtlas));
    } else if(name==='draw_error') {
      show.fire(event());probe.failLayer('clearRect');
      try{hide.fire(event());}catch(error){probe.layerNote(error===probe.layerFailure()?'same':'different');}
      hide.fire(event());snapshot();
    } else if(name==='reentry') {
      const live=event(6,0,3,3);
      probe.reenterLayer('fillRect',()=>Object.assign(live,event(1,1,2,3)));
      show.fire(live);hide.fire(event());snapshot();
    } else if(name==='font_reentry') {
      probe.reenterLayer('beginPath',()=>{
        const d=dimensions();d.device.cell.width=12;d.device.cell.height=22;
        d.device.char.left=2;d.device.char.top=3;
        d.device.canvas={width:96,height:88};d.css.canvas={width:64,height:88/1.5};
        layer.resize(terminal,d);
      });
      const cell=new CellData();cell.setFromCharData([0,'A',1,65]);
      layer._fillCharTrueColor(terminal,cell,1,0);snapshot();
    } else if(name==='base') {
      layer._clearCells(1,1,2,1);layer._ctx.fillStyle='#00ff00';
      const cell=new CellData();cell.setFromCharData([0,'界',2,30028]);
      layer._fillCharTrueColor(terminal,cell,1,0);snapshot();
      probe.layerNote(layer._getFont(terminal,true,true));
      layer._setTransparency(terminal,true);snapshot();
      layer._setTransparency(terminal,true);
      layer._setTransparency(terminal,false);snapshot();
      options.options.theme={foreground:'#ff0000',background:'#112233'};snapshot();
    } else if(name==='transparency_error') {
      probe.failLayer('context');
      try{layer._setTransparency(terminal,true);}catch(error){probe.layerNote(error===probe.layerFailure()?'same':'different');}
      probe.layerNote('children:'+container.children.length);
      layer._setTransparency(terminal,true); // alpha already changed; no retry
    } else throw new Error('Unknown scenario '+name);
    layer.dispose();layer.dispose();
    probe.layerNote('children:'+container.children.length);
    result=probe.endLayerCapture();
  } finally {
    if(layer)layer.dispose();
    if(terminal)removeTerminalFromCache(terminal);
    if(theme)theme.dispose();
    if(options)options.dispose();
    container.remove();
    if(result===undefined)probe.endLayerCapture();
  }
  return result;
}
