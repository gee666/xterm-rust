// Test data and accessor factories shared by the unchanged-source oracle and
// the Rust/WASM assertions. No terminal implementation is provided here.
export const methods={csi:'registerCsiHandler',dcs:'registerDcsHandler',esc:'registerEscHandler',apc:'registerApcHandler'};
const absent={kind:'undefined'};
export function identifierSpecs() {
  const specs=[
    {name:'literal-csi',kind:'csi',literal:true,final:'Z',sequence:'\x1b[Z'},
    {name:'tracked-csi',kind:'csi',prefix:'?',intermediates:'!',final:'Z',sequence:'\x1b[?!Z'},
    {name:'invalid-prefix-before-intermediate-throw',kind:'csi',prefix:'A',intermediates:{kind:'throw'},final:'Z',sequence:'\x1b[Z'},
    {name:'final-changes-after-length-check',kind:'csi',final:['Z','Z',''],sequence:'\x1b[Z'},
    {name:'window-check-z-packed-t',kind:'csi',final:['Z','t','t'],sequence:'\x1b[14t'},
    {name:'window-check-t-packed-z',kind:'csi',final:['t','Z','Z'],sequence:'\x1b[14Z'},
    {name:'reentrant-property-registers-esc',kind:'csi',reentrant:true,final:'Z',sequence:'\x1b[Z\x1bY'},
    {name:'tracked-dcs',kind:'dcs',prefix:'?',intermediates:'!',final:'p',sequence:'\x1bP?1;2!ppayload\x1b\\'},
    {name:'tracked-esc',kind:'esc',intermediates:'(',final:'Z',sequence:'\x1b(Z'},
    {name:'apc-prefix-clears',kind:'apc',prefix:'?',final:'Z',sequence:'\x1b_Zpayload\x1b\\'},
    {name:'apc-setter-keeps-prefix',kind:'apc',keepPrefix:true,prefix:'?',final:'Z',sequence:'\x1b_Zpayload\x1b\\'},
    {name:'apc-read-only-prefix',kind:'apc',readOnly:true,prefix:'?',final:'Z',sequence:'\x1b_Zpayload\x1b\\'},
    {name:'apc-proxy-denies-set',kind:'apc',proxy:true,denySet:true,prefix:'?',final:'Z',sequence:'\x1b_Zpayload\x1b\\'},
    {name:'dcs-intermediate-grows-after-check',kind:'dcs',intermediates:['!!','!!','!!!'],final:'p',sequence:'\x1bP!!!ppayload\x1b\\'},
    {name:'final-getter-throws',kind:'csi',final:['Z',{kind:'throw'}],sequence:'\x1b[Z'},
    {name:'prefix-live-length',kind:'csi',prefix:['?','','?'],final:'Z',sequence:'\x1b[?Z'},
    {name:'intermediate-live-loop-length',kind:'csi',intermediates:['!','!',''],final:'Z',sequence:'\x1b[Z'},
    {name:'prefix-char-code-nan',kind:'csi',prefix:['?','?',''],final:'Z',sequence:'\x1b[Z'},
    {name:'unpaired-final',kind:'csi',final:'\ud800',sequence:'\x1b[Z'},
    {name:'dcs-prefix-length-error-precedence',kind:'dcs',prefix:'??',intermediates:{kind:'throw'},final:'p',sequence:'\x1bPppayload\x1b\\'},
    {name:'esc-low-final',kind:'esc',intermediates:')',final:'0',sequence:'\x1b)0'},
    {name:'prototype-csi',kind:'csi',prototype:true,prefix:'?',intermediates:'!',final:'Z',sequence:'\x1b[?!Z'},
    {name:'proxy-csi',kind:'csi',proxy:true,prefix:'?',intermediates:'!',final:'Z',sequence:'\x1b[?!Z'},
  ];
  for(const kind of Object.keys(methods)) {
    for(const [label,value] of [['absent',absent],['null',null],['false',false],['zero',0],['nan',{kind:'nan'}],['infinity',{kind:'infinity'}],['string','?'],['low',';'],['minimum','<'],['equal','='],['greater','>'],['too-high','@'],['two','??'],['boxed',{kind:'boxed',value:'?'}],['symbol',{kind:'symbol'}],['number',17]]) {
      specs.push({name:`${kind}-prefix-${label}`,kind,prefix:value,final:'Z',sequence:kind==='csi'?'\x1b[?Z':kind==='dcs'?'\x1bP?Zdata\x1b\\':kind==='esc'?'\x1bZ':'\x1b_Zdata\x1b\\'});
    }
    for(const [label,value] of [['empty',''],['minimum',' '],['maximum','/'],['low','\x1f'],['high','0'],['two','!/'],['three','!!!'],['unpaired','\ud800'],['boxed',{kind:'boxed',value:'!'}],['number',17],['false',false]]) {
      specs.push({name:`${kind}-intermediate-${label}`,kind,intermediates:value,final:'Z',sequence:kind==='csi'?'\x1b[Z':kind==='dcs'?'\x1bPZdata\x1b\\':kind==='esc'?'\x1bZ':'\x1b_Zdata\x1b\\'});
    }
    for(const [label,value] of [['empty',''],['before-esc','/'],['esc-low','0'],['before-csi','?'],['csi-low','@'],['upper','~'],['one','Z'],['high','\x7f'],['two','ZZ'],['astral','😀'],['unpaired','\udfff'],['absent',absent],['null',null],['false',false],['number',17],['boxed',{kind:'boxed',value:'Z'}]]) {
      specs.push({name:`${kind}-final-${label}`,kind,final:value,sequence:kind==='csi'?'\x1b[Z':kind==='dcs'?'\x1bPZdata\x1b\\':kind==='esc'?'\x1bZ':'\x1b_Zdata\x1b\\'});
    }
  }
  return specs;
}

export function makeIdentifier(spec,log,hook) {
  const sentinel={identifierSentinel:spec.name};
  const reads={};
  let cleared=false;
  let hooked=false;
  let id;
  function resolve(value) {
    if(value&&typeof value==='object') {
      if(value.kind==='undefined')return undefined;
      if(value.kind==='nan')return NaN;
      if(value.kind==='infinity')return Infinity;
      if(value.kind==='symbol')return Symbol('identifier');
      if(value.kind==='boxed')return new String(value.value);
      if(value.kind==='throw')throw sentinel;
    }
    return value;
  }
  function get(field) {
    log.push(`get:${field}`);
    if(spec.reentrant&&!hooked) {hooked=true;hook();}
    const count=reads[field]??0;reads[field]=count+1;
    let value=spec[field];
    if(Array.isArray(value))value=value[Math.min(count,value.length-1)];
    return field==='prefix'&&cleared?undefined:resolve(value);
  }
  function set(field,value) {
    log.push(`set:${field}:${value===undefined?'undefined':String(value)}`);
    if(field==='prefix'&&!spec.keepPrefix)cleared=true;
  }
  if(spec.literal) {
    id={prefix:resolve(spec.prefix),intermediates:resolve(spec.intermediates),final:resolve(spec.final)};
  } else {
    const storage={};
    for(const field of ['prefix','intermediates','final']) {
      const descriptor={configurable:true,enumerable:true,get(){if(this!==id)log.push('wrong-receiver');return get(field);}};
      if(!(spec.readOnly&&field==='prefix'))descriptor.set=function(value){if(this!==id)log.push('wrong-receiver');set(field,value);};
      Object.defineProperty(storage,field,descriptor);
    }
    id=spec.prototype?Object.create(storage):storage;
    if(spec.proxy)id=new Proxy(id,{
      get(target,key,receiver) {log.push(`proxy-get:${String(key)}`);return Reflect.get(target,key,receiver);},
      set(target,key,value,receiver) {log.push(`proxy-set:${String(key)}`);return spec.denySet?false:Reflect.set(target,key,value,receiver);}
    });
  }
  return {id,sentinel};
}

export function registerIdentifier(api,spec,id,calls) {
  return api[methods[spec.kind]](id,()=>{calls.push('outer');return true;});
}
export function identifierHook(api,calls) {
  return ()=>api.registerEscHandler({final:'Y'},()=>{calls.push('nested');return true;});
}

export function identifierObservation(name,log,calls,failed,error,sentinel) {
  return {name,log:Array.from(log),calls:Array.from(calls),error:failed?observeError(error,sentinel):null};
}

export function observeError(error,sentinel) {
  if(error===sentinel)return {kind:'sentinel'};
  return {kind:error?.name??typeof error,message:error?.name==='Error'?error.message:null};
}
