export function nextRulerFrame() {
  return new Promise(resolve => requestAnimationFrame(resolve));
}

// Record actual native canvas operations while still drawing real browser pixels.
export function recordRulerContext(canvas) {
  const context = canvas.getContext('2d');
  const calls = [];
  const restores = [];
  for (const [name, tag] of [['clearRect','clear'],['fillRect','rect']]) {
    const method = context[name];
    context[name] = function(...args) { calls.push([tag,...args]); return method.apply(this,args); };
    restores.push(() => delete context[name]);
  }
  for (const [name,tag] of [['lineWidth','line'],['fillStyle','style']]) {
    let prototype = Object.getPrototypeOf(context), descriptor;
    while (prototype && !(descriptor=Object.getOwnPropertyDescriptor(prototype,name))) prototype=Object.getPrototypeOf(prototype);
    if (!descriptor?.set || !descriptor?.get) throw new Error(`Missing native ${name} descriptor`);
    Object.defineProperty(context,name,{configurable:true,get(){return descriptor.get.call(this);},set(value){
      calls.push([tag,name==='fillStyle'?Array.from({length:value.length},(_,i)=>value.charCodeAt(i)):value]);
      descriptor.set.call(this,value);
    }});
    restores.push(() => delete context[name]);
  }
  return { calls, restore(){ for(const restore of restores) restore(); } };
}
