# Live touch-field and history checkpoints

This supplements the normal numeric policy contract in `docs/acceptance/touch-policy.md`. Source is `source_repo/src/browser/scrollable/touch.ts`. Do not weaken or regenerate the existing 639 policy traces to accommodate this work. The source's host property reads can invoke getters, so the DOM adapter must not batch or cache reads across source checkpoints.

## TL1. Start

- Capture the batch Date.now before canceling the previous frame and reading targetTouches.
- Snapshot targetTouches.length, then call item for each original index.
- Read the computed identifier key before the new record's fields. Preserve the second identifier read for its stored id field, even though the Rust implementation does not use that field later.
- Read initial target and initial X/Y, then separate first rolling X/Y reads. A getter can return different initial and rolling coordinates. Insert the completed record only after these reads, retaining the original computed key.
- START reads touch.target again after insertion, creates its event, then reads pageX/Y again for event fields. Its target need not equal the record's initial target when accessors change.
- Dispatch one touch before obtaining the next touch item. No active-map guard may survive a host read or event dispatch.

## TL2. Move

- Snapshot batch clock and changedTouches length. Source reads identifier for the own-property test and again to fetch the record; preserve the distinctions when getters change IDs.
- Capture the source record, create CHANGE before reading coordinate deltas, then read each page coordinate and current history tail in source order. Setting translationX happens before obtaining translationY, followed by fresh pageX/Y and clientX/Y reads.
- Dispatch before history mutation. A nested start replaces the map entry but does not redirect the outer record's commit.
- Source first checks rollingPageX length and, if greater than three, shifts X, Y and timestamps separately. It then reads/pushes pageX, reads/pushes pageY, and appends the original batch timestamp.
- A page getter may reenter between those pushes. Independent arrays can temporarily have different lengths, become empty or exceed four entries through nested commits. Preserve actual source state instead of forcing a coherent fixed four-sample tuple. Undefined numeric tails participate as NaN in arithmetic; keep JavaScript field values distinct where exposed.

## TL3. End

- Read the end batch clock, snapshot active count, then snapshot changedTouches length. For each touch preserve separate identifier reads for existence, capture and final deletion.
- Compute hold time through its separate Date.now call before consulting current record fields. Retain source short-circuit order for hold and per-axis distance checks.
- TAP/CONTEXT_MENU fields use fresh tail reads after creating their event. An event-construction hook may change history between classification and event fields.
- Movement captures source velocity operands and insertion-order target snapshot before scheduling. END construction and dispatch precede deletion by a fresh identifier getter. A replacement record can therefore be deleted, or a changed identifier can delete a different record, exactly as source does.

## TL4. Dispatch

- Preserve source event.type getter order and short circuits rather than caching one type for the whole method. Tap clock uses new Date().getTime, not Date.now.
- After the initial Node check, reread event.initialTarget at each containment and depth checkpoint where the source does. A containment hook can replace event fields or register/remove targets while the live linked-list iterator is suspended.
- Preserve depth-sort stability and the completed dispatch snapshot. No record/list RefCell guard crosses host functions, getters, property writes or event dispatch.

## TL5. Failures and partial state

- Getter or host callback failures propagate the original thrown JS value through the listener boundary. Do not translate a failed read into an empty event, clamp values, retry, or discard already completed source mutations.
- Add actual browser accessors that throw or reenter at each source stage. Compare read logs, event fields, cancellation, subsequent deltas, active-record selection and pending frame state to the original implementation.
- Existing normal policy/debug/release cases must remain unchanged and passing. Add deterministic counterexamples and bounded seeded getter-reentry traces; retain full inputs and failing stage.

## TL6. Evidence

Run original touch.ts with controlled hosts to obtain expected operation traces, then run the actual Rust DOM producer on identical inputs in Chromium, Firefox and WebKit. Record exact source hashes, counts, failures and environmental substitutions. Native arithmetic tests alone cannot close this boundary. Keep all current unrun checks marked unrun during the resource pause, and keep overall equivalence NOT PROVEN.
