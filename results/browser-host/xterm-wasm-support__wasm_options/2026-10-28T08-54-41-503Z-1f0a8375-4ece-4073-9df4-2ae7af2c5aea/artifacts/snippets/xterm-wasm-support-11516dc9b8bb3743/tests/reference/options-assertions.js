import {referencePublic} from './options-reference.js';
function equal(a,b,label) {
  if(JSON.stringify(a)!==JSON.stringify(b)) throw Error(`${label}\nexpected ${JSON.stringify(a)}\nactual ${JSON.stringify(b)}`);
}
function normalized(value) {
  const type=typeof value;
  if(value===null)return ['null'];
  if(type==='undefined')return ['undefined'];
  if(type==='number')return ['number',Object.is(value,-0)?'-0':String(value)];
  if(type==='bigint'||type==='symbol'||type==='function')return [type,type==='function'?'function':String(value)];
  if(type!=='object')return [type,value];
  return ['object',Object.keys(value).map(key=>[key,typeof value[key],String(value[key])])];
}
function read(get,key) {
  try{return {value:normalized(get(key))};}
  catch(error){return {error:error.message,name:error.name};}
}
export function compareOptions(reference, referenceContext, actualContext, get, set, layout, publicObject, publicError, operations, eventLog, assign) {
  try {
  const core=reference.options;
  const names=Object.getOwnPropertyNames(core);
  const expectedLayout=names.map(key=>[key,Object.getOwnPropertyDescriptor(core,key).enumerable]);
  equal(layout,expectedLayout,'core accessor layout');
  let expectedPublic, expectedError;
  try{expectedPublic=referencePublic(reference);}catch(error){expectedError=error;}
  for(const key of names) equal(read(get,key),read(key=>core[key],key),`core value ${key}`);
  equal(publicError?{name:publicError.name,message:publicError.message}:null,
        expectedError?{name:expectedError.name,message:expectedError.message}:null,'public constructor failure');
  if(!expectedError) {
    equal(Object.getOwnPropertyNames(publicObject),Object.getOwnPropertyNames(expectedPublic),'public keys');
    for(const key of Object.keys(expectedPublic)) {
      equal(read(key=>publicObject[key],key),read(key=>expectedPublic[key],key),`public value ${key}`);
      const a=Object.getOwnPropertyDescriptor(publicObject,key),b=Object.getOwnPropertyDescriptor(expectedPublic,key);
      equal([a.enumerable,a.configurable,typeof a.get,typeof a.set],[b.enumerable,b.configurable,typeof b.get,typeof b.set],`descriptor ${key}`);
    }
  }
  const referenceEvents=[];
  reference.onOptionChange(key=>referenceEvents.push(key));
  const run=new Function('get','set','view','context','assign',operations);
  const expected=run(key=>core[key],(key,value)=>core[key]=value,expectedPublic,referenceContext,input=>{for(const key in input)expectedPublic[key]=input[key];});
  const actual=run(get,set,publicObject,actualContext,assign);
  equal(actual,expected,'operation result');
  equal(eventLog(),referenceEvents,'change events');
  equal(actualContext.trace,referenceContext.trace,'constructor/getter trace');
  reference.dispose();
  } finally { referenceContext.cleanup?.(); actualContext.cleanup?.(); }
}
