// Component adapter uses the real Rust controller and a real DOM mouseleave.
const reads = [], events = [], marker = {};
const assert = (value, message) => { if (!value) throw new Error(`${mode}: ${message}`); };
let armed = false, startReads = 0, endReads = 0;
const point = end => ({ x: end ? 10 : 1, get y() {
  if (!armed) return 2;
  const ordinal = end ? ++endReads : ++startReads;
  reads.push(`${end ? 'end' : 'start'}:${ordinal}`);
  if (mode === `throw second ${end ? 'end' : 'start'}` && ordinal === 2) throw marker;
  if (mode === `clear ${ordinal === 1 ? 'first' : 'second'} ${end ? 'end' : 'start'}`) {
    element.dispatchEvent(new MouseEvent('mouseleave'));
  }
  if (mode === `${ordinal === 1 ? 'first' : 'second'} ${end ? 'end' : 'start'} outside`) return end ? 3 : 1;
  return 2;
} });
const link = { text: mode, range: { start: point(false), end: point(true) },
  decorations: { underline: false, pointerCursor: false }, activate() {}, hover() {}, leave() { events.push('leave'); } };
return { link, arm() { armed = true; }, verify(error) {
  armed = false;
  if (mode.startsWith('clear') && mode !== 'clear first end') assert(error instanceof TypeError && error.message.includes('link'), 'currentLink reentry TypeError');
  else assert(error === (mode.startsWith('throw') ? marker : undefined), 'exact error identity');
  const expected = mode === 'first start outside' || mode === 'clear first start' ? ['start:1']
    : ['first end outside', 'zero start', 'clear first end'].includes(mode) ? ['start:1', 'end:1']
    : ['second start outside', 'throw second start', 'clear second start'].includes(mode) ? ['start:1', 'end:1', 'start:2']
    : ['start:1', 'end:1', 'start:2', 'end:2'];
  assert(JSON.stringify(reads) === JSON.stringify(expected), `range reads ${JSON.stringify(reads)}`);
  assert(events.join() === (['plain', 'zero start'].includes(mode) || mode.startsWith('clear') ? 'leave' : ''), 'leave phase');
} };
