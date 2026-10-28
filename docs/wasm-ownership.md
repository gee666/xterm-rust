# Browser ownership and JavaScript GC

Status: unresolved implementation work. Explicit dispose is not proof that unreachable JavaScript cycles are collectible. Do not mark the WASM lifetime criterion complete from the component tests alone.

## Concrete cycles to test

The browser options adapter can retain a logger, link handler or arbitrary object whose closure captures the public Terminal. The current chain can be Rust Shared -> OptionsBinding or OptionsService -> JsValue raw object -> JavaScript closure -> public Terminal -> Rust Shared. JavaScript cannot trace Rust Rc ownership, so source JavaScript cycle collection does not automatically apply.

The same risk applies to parser/event callbacks and addons which retain their activated terminal. An extracted event registration function or options object has the opposite requirement: it must stay usable for as long as source keeps its core alive. Replacing every strong reference with Weak would prematurely destroy live source state.

DOM lifetime must also be checked. A terminal element attached to document.body and its registered input handlers can keep the source CoreBrowserTerminal alive even after the public wrapper is no longer referenced. A detached terminal whose only references are within a JavaScript cycle should be collectible. Weak Rust listeners alone avoid one cycle but do not prove both source lifetimes.

## Proposed ownership boundary to implement and review

Keep JS-facing owners in the small browser facade, where the JavaScript collector can trace them. The facade owns its raw options object, public option view, event registration functions, addon instances and DOM nodes. Rust retains WeakRef references to those host owners when a strong externref would close a cycle. JavaScript-owned callback functions may hold a Rust state handle when source requires that lifetime, but that Rust state must not strongly root the same JavaScript owner.

A retained public options view may need to retain the core/browser state, while the Rust state holds only a weak reference to that JS view. A retained registration function has analogous requirements. Constructor-time raw objects must not disappear between reads, so each host operation should upgrade its weak owner into a temporary strong local for the duration of the operation. Do not hold RefCell or mutable host borrows across user callbacks.

This is a proposal, not permission to change the contract silently. Review it against source, retain explicit disposal behavior and document any representation changes. Do not use Closure::forget, leaked Rc cycles or weak callbacks that silently disappear.

## Required evidence

Use built Rust WASM and the public facade in Chromium, Firefox and WebKit. Run source and Rust with equivalent operations. Use supported GC instrumentation where available and report unsupported deterministic collection checks rather than assuming they pass.

- Logger, linkHandler, options object, event callback, parser callback and addon each capture their terminal. Test explicit dispose, repeated dispose and unreachable cycles.
- Retain only terminal.options; read/write options after the wrapper becomes unreachable. Check dynamic getters, callback errors, linked service effects and source disposal behavior.
- Retain only an event registration function, an active subscription or an addon. Verify which source objects remain alive and which can be collected.
- Keep a terminal DOM root attached while dropping the public wrapper, then exercise input/rendering. Detach it and release all outside references; verify listener/state cleanup.
- Remove listeners or dispose from inside callbacks, including recursive dispatch and thrown values. No alias panic, double drop or invalid closure invocation is allowed.
- Retained callback errors preserve the original JsValue identity through every Rust error source chain.

Store graph diagrams, instrumentation commands, exact browser versions, passes/failures/unrun counts and retained counterexamples under results/browser-lifetime/. A GC test timeout is a recorded result, not evidence that collection is impossible or that source and Rust are equivalent.
