# Preliminary independent review

The isolated swarm_review session found no confirmed implementation defect.
It checked three EXE pins, executed all24 native tests with24 passes, syntax-checked
both new JS files and both scripts, and inspected tests, docs and lint/fmt logs.
No builds or browsers ran and no files were modified by the reviewer.

Verdict changes requested. Missing unchanged-source5×3 fixtures/drift checks,
actual Rust5×3, source/compiler/artifact pins, old/fixed ABI controls, exact source
restoration/compiler identity and regression matrices prevent acceptance.
The public documentation must explicitly mark those checks unrun until supplied.
Raw registries/nonstring-name parity remain outside490 and open under305.
