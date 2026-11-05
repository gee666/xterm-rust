
export function localizableStrings(pg, ps, og, os) {
  return {
    get promptLabel() { return pg(); },
    set promptLabel(value) { ps(value); },
    get tooMuchOutput() { return og(); },
    set tooMuchOutput(value) { os(value); }
  };
}
