# Core browser service acceptance

Reference is `source_repo/src/browser/services/CoreBrowserService.ts`, including its private ScreenDprMonitor, and the ICoreBrowserService declaration in `services/Services.ts`. This is a browser component, not a headless substitute. All source methods and observable ordering below require Rust-backed browser assertions and unchanged-source observations.

## CB1. Focus and microtask cache

- Constructor starts `_isFocused=false` and cache undefined. Register textarea focus then blur listeners after DPR monitor and forwarding setup, retaining source order.
- Focus sets the stored flag true; blur sets it false. Neither event invalidates an already-computed cache within the same microtask turn.
- On an uncached getter, short-circuit false without calling ownerDocument.hasFocus. When stored focus is true, query the textarea's current ownerDocument, not the constructor mainDocument or current rendering window.
- Cache the Boolean result before queueing the global microtask. Multiple reads in one turn return that result without further hasFocus or scheduling calls. The microtask clears the cache. Focus/blur transitions within that turn remain intentionally stale.
- Preserve host getter/call exceptions and source partial state. An enqueue failure leaves the computed cache committed. A throwing hasFocus call leaves it unset. Original thrown JS values must survive the Rust error boundary.
- Explicit service disposal removes DOM listeners but does not invent a false focus result or cancel a source-already-queued cache invalidation. Dropping the final Rust owner must not leave a callable dangling WASM closure.

## CB2. Window, DPR and media query state

- Retain supplied textarea, parent window and mainDocument identities separately. mainDocument is readonly and does not change when the rendering window changes.
- Window setter compares JS object identity. Same object does nothing. A different object is committed before onWindowChange fires, including when a listener fails. Listeners may query the committed window and reenter without held Rust borrows.
- `dpr` is the current window's raw devicePixelRatio, not a cached/defaulted/clamped value. Include zero, negative zero, fractional, NaN and infinity values under controlled host getters.
- Initialize the monitor's current DPR, install its legacy media query listener, then install its resize listener. Source uses addListener/removeListener, not addEventListener('change').
- A window switch assigns the monitor parent, creates the new resize listener before replacing/disposal of the old one, then compares DPR. Keep add/remove/error ordering visible.
- Compare with JavaScript strict Number inequality: NaN differs from itself; positive and negative zero compare equal. Fire the new DPR before updating the stored DPR or rearming the query. A callback may mutate the window/DPR, dispose, or throw; subsequent operations must follow source ordering.
- Rearm even when DPR is unchanged: remove old media listener, read/set current DPR, read DPR again for `screen and (resolution: Ndppx)`, call matchMedia, then add the same callback identity. Do not collapse getter reads or reorder current-value assignment around host calls.
- Format N with ECMAScript Number formatting. Existing ryu-js is already a dependency; no new formatter/dependency is justified.
- clearListener has the exact no-list/no-callback early return. Otherwise remove the listener before clearing list and callback. Resize and DPR observers must not continue after successful explicit disposal.

## CB3. Errors, ownership and real integration

- Propagate matchMedia, add/remove listener, ownerDocument/hasFocus and subscriber errors as the original JS thrown value. Do not convert errors into successful defaults, swallow failures, or panic from untrusted host operations.
- Release RefCell borrows before any JS operation or callback. Use reentrant Fn event callbacks, not FnMut aliases that trap on recursive dispatch.
- Distinguish successful explicit disposal, partial failure and Rust Drop cleanup. Document any unavoidable language-specific resource cleanup difference rather than claiming equivalent exception behavior.
- Keep media callback identity stable across query rearming and window replacement. Retain ownership until removal or a safe JS lifetime transfer; no Closure::forget or reference cycle.
- Connect this same live service to the terminal, DOM renderer, character-size service and render scheduler. Different detached window/focus/DPR snapshots do not satisfy integration.
- Test initial creation, focus/cache turns, both event paths, window identity/no-change/switch, equal/different/NaN DPR, reentrant change/error/disposal, legacy listener order and post-dispose queries.
- Execute source and Rust-backed component assertions in Chromium, Firefox and WebKit. Record discovered/executed/pass/fail/skip/unrun counts, source/code hashes, exact commands and engine versions. A compile or a native fake is not browser execution evidence.
- Keep files under350 lines, use safe Rust and existing dependencies, preserve source MIT attribution, pass scoped fmt/Clippy/WASM builds and obtain independent review. Native Linux/macOS and full terminal equivalence remain separate gates.
