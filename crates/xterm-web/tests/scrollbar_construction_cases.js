// Shared full-Terminal scenarios. Source and Rust execute these synchronously,
// before any scheduled animation frame can add renderer option reads.
export const scrollbarExpected = [
  { reads: ['showArrows','showScrollbar','showArrows','width','showScrollbar','width','showScrollbar','width','showScrollbar','width'], ruler: true, error: false },
  { reads: ['showArrows','showScrollbar','showArrows','showScrollbar','width'], ruler: false, error: false },
  { reads: ['showArrows','showScrollbar','showArrows','showScrollbar','width'], ruler: false, error: true },
  { reads: ['showArrows','showScrollbar','showArrows','width','showScrollbar','width'], ruler: false, error: false },
  { reads: ['showArrows','showScrollbar','showArrows','width','showScrollbar','width'], ruler: false, error: false },
  { reads: ['showScrollbar','showArrows','showScrollbar'], ruler: false, error: false },
  { reads: ['showArrows','showScrollbar','showArrows','width','showScrollbar'], ruler: false, error: true },
  { reads: ['showArrows','showScrollbar','showArrows','width','showScrollbar','width'], ruler: false, error: false }
];
export function scrollbarScenario(index) {
  const reads = [], sentinel = Symbol('scrollbar width/getter failure');
  let visible = index !== 1 && index !== 2, widths = 0, shows = 0;
  const scrollbar = {
    get showArrows() { reads.push('showArrows'); return false; },
    get showScrollbar() {
      reads.push('showScrollbar');
      if (index === 6 && ++shows === 2) throw sentinel;
      return visible;
    },
    get width() {
      reads.push('width');
      if (index === 2) throw sentinel;
      if (index === 7) visible = false;
      return index === 3 || index === 5 || (index === 4 && widths++ > 0) ? 0 : 18;
    }
  };
  const parent = document.createElement('div');
  document.body.appendChild(parent);
  return { index, reads, sentinel, parent,
    options: { cols: 8, rows: 3, scrollback: 20, scrollbar },
    update: { get showArrows() { reads.push('showArrows'); return false; },
      get showScrollbar() { reads.push('showScrollbar'); return false; },
      get width() { reads.push('width'); throw sentinel; } } };
}
export function scrollbarCapture(scenario, error) {
  if (error !== undefined && error !== scenario.sentinel) throw error;
  return { reads: [...scenario.reads],
    ruler: !!scenario.parent.querySelector('.xterm-decoration-overview-ruler'),
    error: error === scenario.sentinel };
}
export function scrollbarSourceCase(Terminal, index) {
  const scenario = scrollbarScenario(index);
  const terminal = new Terminal(scenario.options);
  let error;
  try {
    terminal.open(scenario.parent);
    if (index === 5) {
      scenario.reads.length = 0;
      terminal.options.scrollbar = scenario.update;
    }
  } catch (failure) { error = failure; }
  try { return scrollbarCapture(scenario, error); }
  finally { terminal.dispose(); scenario.parent.remove(); }
}
