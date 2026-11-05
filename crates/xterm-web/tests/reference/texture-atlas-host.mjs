// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Real contexts with temporary method/setter interception. No terminal fallback.
export function atlasHostPhase(kind,specText,rustInvoke) {
  const spec=JSON.parse(specText),atlas=rustInvoke?null:globalThis.createSourceAtlas(spec);
  const token=Symbol('atlas host error'),trace=[];
  const imageData=Object.getOwnPropertyDescriptor(globalThis,'ImageData');
  const proto=CanvasRenderingContext2D.prototype,saved=new Map();
  const remember=name=>{const descriptor=Object.getOwnPropertyDescriptor(proto,name);saved.set(name,descriptor);return descriptor;};
  let fills=0,failed=false;
  const invoke=rustInvoke??(()=>{
    const glyph=atlas.getRasterizedGlyph(65,0,kind==='invisible'?0x40000000:0,0,false,undefined);
    return JSON.stringify(globalThis.atlasGlyphSnapshot(glyph));
  });
  let identity=false,retry;
  try {
    if(kind==='imageCtor') {
      globalThis.ImageData=new Proxy(ImageData,{construct(target,args){trace.push('new ImageData');if(!failed){failed=true;throw token;}return Reflect.construct(target,args);}});
    }
    for(const name of ['save','restore','fillRect','fillText','putImageData']) {
      const original=remember(name);
      if(kind==='method'&&name==='fillText') {
        Object.defineProperty(proto,name,{configurable:true,get(){trace.push('get fillText');if(!failed){failed=true;throw token;}return original.value;}});
      } else Object.defineProperty(proto,name,{configurable:true,writable:true,value:function(...args){
        trace.push([name,args.length]);
        if(kind==='upload'&&name==='putImageData'&&!failed){failed=true;throw token;}
        return original.value.apply(this,args);
      }});
    }
    for(const name of ['globalCompositeOperation','fillStyle','font','textBaseline']) {
      const original=remember(name);
      Object.defineProperty(proto,name,{configurable:true,get:original.get,set(value){
        trace.push([name,value]);
        if(name==='fillStyle'&&++fills===2&&kind==='style'&&!failed){failed=true;throw token;}
        return original.set.call(this,value);
      }});
    }
    try {invoke();}catch(error){identity=error===token;}
    retry=JSON.parse(invoke());
  } finally {
    for(const [name,descriptor] of saved)Object.defineProperty(proto,name,descriptor);
    Object.defineProperty(globalThis,'ImageData',imageData);
    atlas?.dispose();
  }
  return JSON.stringify({trace,identity,retry});
}
