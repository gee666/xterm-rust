// Ownership glue only; focus/cache decisions remain in Rust.
export function queueFocusCache(run) {
  let invoked = false;
  const invoke = cancelled => {
    if (invoked) return;
    invoked = true;
    run(cancelled);
  };
  try {
    queueMicrotask(() => invoke(false));
  } catch (error) {
    // An enqueue hook may accept the native job and then throw. Release only
    // after that already-queued job has had its FIFO turn. An intrinsic async
    // checkpoint also releases failed-before-enqueue callbacks without WeakRef.
    void (async () => { await 0; invoke(true); })();
    throw error;
  }
}
