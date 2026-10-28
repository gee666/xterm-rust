
export function copyOptionsDefaults(template) { return {...template}; }
export function initializeOptions(input, defaults, apply) {
  for (const key in input) {
    if (key in defaults) {
      try { defaults[key] = apply(key, input[key]); }
      catch (error) { console.error(error); }
    }
  }
  return defaults;
}
export function optionMembership(template, key) { return key in template; }
export function optionStrictEqual(left,right) { return left === right; }
export function optionLayout(raw) {
  const entries = [];
  for (const key in raw) entries.push([key, Object.prototype.hasOwnProperty.call(raw, key)]);
  return entries;
}
export function defineOption(target, key, enumerable, get, set) {
  Object.defineProperty(target, key, {get, set, enumerable, configurable:enumerable});
}
export function publicOptions(core, checkReadonly) {
  const view = {...core};
  const getter = key => core[key];
  const setter = (key,value) => { checkReadonly(key); core[key]=value; };
  for (const key in core) Object.defineProperty(view,key,{get:getter.bind(null,key),set:setter.bind(null,key)});
  return view;
}
export function assignOptions(target, input) {
  for (const key in input) target[key] = input[key];
}
