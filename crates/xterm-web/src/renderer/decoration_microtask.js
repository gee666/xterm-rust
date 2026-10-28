// Scheduling glue only. Decoration indexing and all terminal logic stay in Rust.
export function queueDecorationSync(run) {
  let invoked = false;
  const invoke = cancelled => {
    if (invoked) return;
    invoked = true;
    return run(cancelled);
  };
  try {
    queueMicrotask(() => {
      const error = invoke(false);
      if (error !== undefined) throw error;
    });
  } catch (error) {
    // An enqueue failure must release the one-shot Rust callback too.
    invoke(true);
    throw error;
  }
}
