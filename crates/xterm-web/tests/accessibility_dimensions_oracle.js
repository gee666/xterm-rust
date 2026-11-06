// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Generated unchanged method bodies. Test-only, never imported by the library.
// AccessibilityManager.ts SHA-256: 6519cc3ee85382a7da9b42a112010044f78f3a1cd4d02fe4fcb37bea36f526ec
export class SourceDimensions {
_refreshRowsDimensions() {
    if (!this._renderService.dimensions.css.cell.height) {
      return;
    }
    Object.assign(this._accessibilityContainer.style, {
      width: `${this._renderService.dimensions.css.canvas.width}px`,
      fontSize: `${this._terminal.options.fontSize}px`
    });
    if (this._rowElements.length !== this._terminal.rows) {
      this._handleResize(this._terminal.rows);
    }
    for (let i = 0; i < this._terminal.rows; i++) {
      this._refreshRowDimensions(this._rowElements[i]);
      this._alignRowWidth(this._rowElements[i]);
    }
  }
_refreshRowDimensions(element) {
    element.style.height = `${this._renderService.dimensions.css.cell.height}px`;
  }
_alignRowWidth(element) {
    element.style.transform = '';
    const width = element.getBoundingClientRect().width;
    const lastColumn = this._rowColumns.get(element)?.slice(-1)?.[0];
    if (!lastColumn) {
      return;
    }
    const targetWidth = lastColumn * this._renderService.dimensions.css.cell.width;
    element.style.transform = `scaleX(${targetWidth / width})`;
  }
}

// MIT. Shared DOM interventions run unchanged source methods and Rust separately.

function scenario(container, mode, refresh, change) {
  const rows=[...container.querySelectorAll('[role=listitem]')];
  const trace=[];
  const cleanup=[];
  const marker={mode};
  let fired=false;
  if(mode==='row-stop')change({cellHeight:20});
  function hook(node,label) {
    const style=node.style;
    const proxy=new Proxy(style,{
      get(target,key) {
        const value=Reflect.get(target,key,target);
        return typeof value==='function'?value.bind(target):value;
      },
      set(target,key,value) {
        trace.push(`${label}.${String(key)}=${value}`);
        if(label==='container'&&key==='width'&&!fired) {
          if(mode==='width-throw')throw marker;
          if(mode==='width-mutate') {fired=true;change({cellHeight:24,fontSize:20});}
        }
        if(label==='row0'&&key==='height'&&!fired) {
          if(mode==='row-throw')throw marker;
          if(mode==='row-stop') {fired=true;change({cellHeight:30,rows:1});}
          if(mode==='height-mutate') {fired=true;change({cellHeight:24});}
        }
        return Reflect.set(target,key,value,target);
      }
    });
    Object.defineProperty(node,'style',{configurable:true,get() {
      trace.push(`${label}.style`);
      if(label==='container'&&!fired) {
        if(mode==='style-throw')throw marker;
        if(mode==='style-get') {fired=true;change({canvasWidth:128,cellHeight:24,fontSize:18});}
      }
      return proxy;
    }});
    cleanup.push(()=>{delete node.style;});
  }
  hook(container,'container');
  rows.forEach((row,index)=>hook(row,`row${index}`));
  let failure='none';
  try {refresh();} catch(error) {failure=error===marker?'marker':`${error.name}:${error.message}`;}
  finally {for(const clean of cleanup)clean();}
  return {trace,failure,width:container.style.width,fontSize:container.style.fontSize,
    heights:rows.map(row=>row.style.height)};
}

export function sourceDimensionsCase(original,mode) {
  const container=original.cloneNode(true);
  original.parentNode.appendChild(container);
  const rows=[...container.querySelectorAll('[role=listitem]')];
  const state={cellHeight:16,cellWidth:8,canvasWidth:64,fontSize:12,rows:3};
  const source=new SourceDimensions();
  source._accessibilityContainer=container;
  source._rowElements=rows;
  source._rowColumns=new WeakMap(rows.map((row,index)=>[row,[0,[5,1,4][index]]]));
  source._terminal={get rows(){return state.rows;},options:{get fontSize(){return state.fontSize;}}};
  source._renderService={get dimensions(){return {css:{cell:{height:state.cellHeight,width:state.cellWidth},canvas:{width:state.canvasWidth}}};}};
  source._handleResize=()=>{throw new Error('Unexpected resize outside the scoped row loop');};
  try {return scenario(container,mode,()=>source._refreshRowsDimensions(),delta=>Object.assign(state,delta));}
  finally {container.remove();}
}
export function rustDimensionsCase(container,mode,refresh,change) {
  return scenario(container,mode,refresh,change);
}
