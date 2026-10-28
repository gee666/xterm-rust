# Scrollbar helpers in progress

Task 165 owns scrollbar_state.rs, visibility.rs, fast_dom_node.rs, browser_visibility.rs and widget.rs under crates/xterm-web/src/scroll. Geometry, visibility, FastDom, browser wrapper and Widget composition are written. Three geometry and seven visibility native tests pass. Source differential and three-browser tests remain unfinished. No component or terminal scrolling equivalence is claimed.

Arrow ownership was explicitly transferred to Bharata, task 166, before arrow.rs was written. That task also owns pointer_monitor.rs and abstract/oriented scrollbar DOM. Norman owns task 71/167 model, viewport, raw options, mouse normalization and root scroll/mod.rs exports. Xavier owns browser crate root exports.

## Geometry contract

ScrollbarState follows the six-number source constructor and snake_case source setters/getters. Constructor visible/scroll/position values are not eagerly rounded. The scrollbar/opposite/arrow sizes use Math.round semantics, including negative zero and ties toward positive infinity. Source NaN propagation is retained instead of Rust f64::max's NaN-eliding behavior. Minimum slider size is the source constant 20, even when it exceeds the available track and creates a negative ratio.

set_opposite_scrollbar_size and set_scrollbar_size do not refresh computed values. Clone reconstructs/recomputes as source does. Desired offsets/deltas/pages are not clamped in this helper; Scrollable owns clamping. The three directed native tests passed. This is not a substitute for source differential and real browser validation.

## Visibility and DOM contracts

VisibilityController<S: TimerScheduler> is the portable class/timer policy. It supports unknown raw visibility values through the shared ScrollbarVisibility representation, not string/number coercion into the three known constants. Reentrant class setters run after all state borrows drop. A reveal commits visible before scheduling zero-delay work. The timeout clears its pending token before the class write. Hiding uses source fade rules; disposal closes the timer without inventing a terminal-state reset.

BrowserVisibility/new_visibility owns the actual Rust Closure until cancellation/drop and uses Cameron's reusable BrowserTimerScheduler. The supplied Window must be the source module's ambient timer realm, not automatically the node's ownerDocument. Explicit host errors propagate as their actual JsValue; disposed-timer errors retain source wording.

FastDomNode::new(HtmlElement) exposes dom_node and accepts JsValue, portable OptionValue, f64 and string callers through DomValue. Rust owns change caches and numeric-only px policy. Non-numbers remain raw until DOM string coercion. Clones share cache identity. Cache writes precede host assignment, as source does, including its retry behavior after a setter throws. A private JS value mirror preserves RHS reads after a reentrant style getter. Inline JavaScript performs only primitive strict property assignments, never terminal/rendering policy.

begin_style/assign_style/set_style_with preserve the source's style-getter-before-RHS-conversion order. Arrow default-hint size concatenation and vertical template string conversion are deliberately different caller operations. Widget must normalize metadata with the shared StandardMouseEvent/IframeCache and register real disposable DOM listeners.

## Remaining verification

- Source-derived geometry mutation/clone/boundary traces, including NaN, infinities, signed zero, tiny values and fractional constructor-versus-setter behavior.
- Deterministic visibility scheduling, cancellation, replacement target, disposal, host error and callback reentry assertions against unchanged JS.
- Actual FastDom style/class write counts, raw value/string coercion, invalid CSS, setter errors, reentrant getters, toggles and lone UTF-16 input through supported browser engines.
- Widget mouse metadata, iframe/adoption, callback order and cleanup using source-derived original methods.
- Native fmt/Clippy/tests plus actual Chromium/Firefox/WebKit component and integrated Viewport checks. Written files and exposed APIs are not execution evidence.

This helper is not a substitute for tasks 166/167/71 or full browser scrolling/input integration. Overall port equivalence remains NOT PROVEN.
