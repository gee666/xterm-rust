# Static review of live mouse reads

Review of task172 current mouse_report/read/values/touch modules against unchanged
MouseService.ts and Mouse.ts. These are source-order findings, not newly executed
runtime failures. Resource preflight is currently below1GiB. Keep17 WASM cases
unrun until a permitted current build and browser run succeeds.

## Sound paths inspected

- overrideType precedes type and preserves truthy short circuit.
- unknown event kinds stop before button/modifier reads.
- move buttons branches resample the source property at each mask branch.
- custom wheel veto precedes delta/metrics; height and DPR arguments precede the
  consume method's second delta and Shift checks.
- source +0 denominator normalization is retained.
- requireAlt and stripAlt are separate live reads, and wheel does not strip Alt.
- preventDefault/stopPropagation and getModifierState use caught Reflect.apply.
- output/cache commit reuses the shared policy instead of a second encoder.

## A: scalar sensitivity conversion is too early

MouseReadHost returns f64 for both sensitivities. The real EngineInputHost reads
an OptionValue and converts it before returning. Source evaluates
`amount * options.fastScrollSensitivity * options.scrollSensitivity` left to
right. It gets the fast value first, then converts the left amount, then converts
the fast value. A number-like object is retained by OptionsService validation.

For an amount object whose valueOf logs or throws, and a fast object whose
valueOf logs or throws, expected order is fast property read, amount coercion,
fast coercion, scroll property read, scroll coercion. Current numeric host API
can coerce fast before amount. Distinct thrown sentinels expose the error.
Use raw host values through the multiplication boundary, or retain this as an
explicit unclosed source path. Do not count primitive-only host traces as proof.

## B: touch compound-assignment prefix is lost on getter reentry

Source `_touchScrollAccumulator -= event.translationY` reads the old accumulator
before evaluating/coercing translationY. The adapter currently reads and coerces
translationY first, then calls consume_touch, which reads the current accumulator.

Directed case: accumulator5, translationY getter dispatches gesturestart to reset
it to0 and returns2, cell height20. Source commits3; current order commits-2.
A throwing getter must leave its own side effects but must not commit the outer
subtraction. Capture the accumulator without a held borrow before the getter,
then commit using that captured value. Add an actual ordered source/WASM case.

## C: raw fallback button conversion precedes source semantics

raw_button converts the second property result with `v|0` immediately. Source
stores that raw result, performs `but > WHEEL`, then later performs strict
button comparisons and bitwise encoding. Changing getters can return0 first,
then2147483648: source rejects >4, while early ToInt32 wraps it negative and can
admit a report. Fractional values can also be distinct in move debouncing even
when they have the same low bits. Number-like objects add coercion/error points.

DOM signed-short assumptions do not cover structural overrideType and overridden
getters that the source intentionally reads dynamically. Keep a raw compatible
button through the relevant checks, or track a precise followup rather than
claiming arbitrary raw getter parity from the enum boundary.

## D: DOM geometry exceptions and expression order

report_coords calls getBoundingClientRect and rect.left/top through uncaught
web-sys methods/getters. Overriding the public element method or returning a
valid DOMRect-like object with a throwing getter can throw past Rust guards.
The event getter path is caught, but the geometry path is not equivalently
caught. Propagate the original thrown value with Result.

For `event.clientX - rect.left`, source obtains both values before numeric
coercion. Calling values::number(clientX) coerces clientX before fetching
rect.left. A valueOf and a rect getter with distinct effects/throws distinguish
these orders. Metadata may narrow render-object proxy claims, but do not omit
public DOM method/getter exceptions under a general throwing-getter claim.

## E: integrated public and RIS reset

The source browser reset calls mouse.reset in addition to core reset. Current
public browser operations.reset only invokes native.reset, and the observed
RequestReset browser path clears selection/decorations/rendering without a
mouse reset. InputBindings.reset_mouse exists but was not called there.

Verify actual Terminal.reset and RIS clear last-event, wheel and touch state,
retain the custom key handler as source does, reset selection/decorations, sync
rows/cols options and refresh. A direct MousePolicy.reset test is not this
integration proof. Xavier and Norman were notified separately.

## Raw callback receiver

The real Root custom wheel dispatch currently calls a handler with undefined as
receiver. Source uses MouseStateService as this. Use a cached live service view,
not an empty object. Direct source calls also ignore a poisoned Function.call
property, so Reflect.apply is required here. Source Event.ts deliberately uses
listener.fn.call instead; do not rewrite all callback invocation sites blindly.
Raw/prototype/full receiver compatibility remains task171, not a typed57 waiver.
