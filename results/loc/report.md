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
| production-library | 99 | 12282 | 11331 | 423637 |
| addons | 15 | 1564 | 1412 | 51751 |
| bindings-glue | 1 | 87 | 75 | 2275 |
| generated-code-data | 9 | 6666 | 6618 | 171892 |
| mixed-code-data | 0 | 0 | 0 | 0 |
| tests | 68 | 10300 | 9854 | 355691 |
| test-data | 32 | 87721 | 87721 | 28599729 |
| demos | 1 | 82 | 80 | 3163 |
| tooling | 51 | 5366 | 5245 | 281179 |
| configuration | 11 | 843 | 824 | 2091804 |
| total | 287 | 124911 | 123160 | 31981121 |

Original inventory SHA-256: b3111ec254ab80a839b69d6b2ee6946a87fac9071be0e8d086feee5ce83fd0f4

Port inventory SHA-256: 906986be26de824ea7d8b952637a1f7a8784880d3a244225d9c5e84b8b5fab8c

Rules SHA-256: b7775c5afd52999e0fb62ae43e0eeaba4d7eb65b5f97e2f405e0eff9e94dc397

Legacy pre-launch subtotal matches: true.

Raw inventories and method cross-check: source.json, port.json, summary.json.
