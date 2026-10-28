# Partial-port source line snapshot

Partial-port working-tree snapshot. Not evidence of equivalent-functionality code reduction.

UTF-8 physical LF/CRLF lines and nonblank lines, including comments; no comment-stripped SLOC.

Tool: xterm-rust count-loc.mjs v1, Node v24.15.0.

## Original

| Category | Files | Physical | Nonblank | Bytes |
| --- | ---: | ---: | ---: | ---: |
| production-library | 115 | 28744 | 25684 | 1060584 |
| addons | 75 | 15738 | 13962 | 747319 |
| bindings-glue | 16 | 4823 | 4321 | 160464 |
| generated-code-data | 0 | 0 | 0 | 0 |
| mixed-code-data | 3 | 528 | 493 | 26581 |
| tests | 127 | 38811 | 35807 | 1699412 |
| test-data | 153 | 3643 | 2533 | 102770 |
| demos | 25 | 4116 | 3720 | 165518 |
| tooling | 35 | 3370 | 3039 | 102899 |
| configuration | 86 | 3198 | 3172 | 80863 |
| total | 635 | 102971 | 92731 | 4146410 |

## Rust port working tree

| Category | Files | Physical | Nonblank | Bytes |
| --- | ---: | ---: | ---: | ---: |
| production-library | 309 | 43632 | 41746 | 1535412 |
| addons | 57 | 7564 | 7209 | 263225 |
| bindings-glue | 2 | 100 | 85 | 2786 |
| generated-code-data | 14 | 8759 | 8705 | 230164 |
| mixed-code-data | 0 | 0 | 0 | 0 |
| tests | 303 | 47715 | 46389 | 1657025 |
| test-data | 101 | 284926 | 284886 | 115307775 |
| demos | 13 | 1386 | 1345 | 56151 |
| tooling | 184 | 18234 | 17978 | 1072872 |
| configuration | 10 | 1471 | 1450 | 42153 |
| total | 993 | 413787 | 409793 | 120167563 |

Original inventory SHA-256: b3111ec254ab80a839b69d6b2ee6946a87fac9071be0e8d086feee5ce83fd0f4

Port inventory SHA-256: d4f8c23e16fc7a3b7584419899f8087441c28b1553624c8ed0fb3cfbe7e16c71

Rules SHA-256: eb18efd109cd9bdef1b27c87a975a10f3af43ea403ea4840dff03a0029f8ef13

Legacy pre-launch subtotal matches: true.

Raw inventories and method cross-check: source.json, port.json, summary.json.
