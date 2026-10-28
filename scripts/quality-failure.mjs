import { isProxy } from 'node:util/types';

/** Describe thrown values without invoking their getters or custom formatting.
 * Empty messages stay empty; callers must classify failure by state, not truthiness. */
export function failureText(value) {
  if (value === null) return 'null';
  if (typeof value === 'string') return value;
  if (typeof value !== 'object' && typeof value !== 'function') return typeof value === 'bigint' ? `${value}n` : String(value);
  const seen = new Set();
  for (let current = value; current !== null && seen.size < 16; current = Object.getPrototypeOf(current)) {
    if (isProxy(current)) return 'Thrown Proxy value, not inspected';
    if (seen.has(current)) return 'Thrown value with cyclic prototype metadata';
    seen.add(current);
    const descriptor = Object.getOwnPropertyDescriptor(current, 'message');
    if (descriptor) return Object.hasOwn(descriptor, 'value') && typeof descriptor.value === 'string'
      ? descriptor.value : 'Thrown value message accessor, not evaluated';
  }
  return typeof value === 'function' ? 'Non-Error function thrown' : 'Non-Error object thrown';
}
