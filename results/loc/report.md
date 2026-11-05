# Partial-port source line snapshot

Partial-port working-tree snapshot. Not evidence of equivalent-functionality code reduction.

UTF-8 physical LF/CRLF lines and nonblank lines, including comments; no comment-stripped SLOC.

Tool: xterm-rust count-loc.mjs v1, Node v24.15.0.

## Original

| Category | Files | Physical | Nonblank | Bytes |
| --- | ---: | ---: | ---: | ---: |
| production-library | 114 | 28590 | 25533 | 1056697 |
| addons | 75 | 15738 | 13962 | 747319 |
| bindings-glue | 16 | 4823 | 4321 | 160464 |
| generated-code-data | 1 | 154 | 151 | 3887 |
| mixed-code-data | 3 | 528 | 493 | 26581 |
| mixed-code-tests | 0 | 0 | 0 | 0 |
| tests | 127 | 38811 | 35807 | 1699412 |
| test-data | 153 | 3643 | 2533 | 102770 |
| demos | 25 | 4116 | 3720 | 165518 |
| tooling | 35 | 3370 | 3039 | 102899 |
| configuration | 86 | 3198 | 3172 | 80863 |
| total | 635 | 102971 | 92731 | 4146410 |

## Rust port working tree

| Category | Files | Physical | Nonblank | Bytes |
| --- | ---: | ---: | ---: | ---: |
| production-library | 385 | 53579 | 51654 | 1879555 |
| addons | 87 | 12601 | 12150 | 437478 |
| bindings-glue | 22 | 2540 | 2445 | 95245 |
| generated-code-data | 17 | 9144 | 9081 | 243621 |
| mixed-code-data | 0 | 0 | 0 | 0 |
| mixed-code-tests | 45 | 7657 | 7340 | 265485 |
| tests | 505 | 77292 | 75420 | 2740369 |
| test-data | 151 | 458430 | 458333 | 142161593 |
| demos | 11 | 983 | 959 | 42283 |
| tooling | 268 | 25944 | 25617 | 1577643 |
| configuration | 11 | 822 | 796 | 50925 |
| total | 1502 | 648992 | 643795 | 149494197 |

Original inventory SHA-256: 319645846cc7e05738b9283d15a6497ec88bd2c9cb723e25c6986122ca533ece

Port inventory SHA-256: a1ba77810ba1417af638a3d7fe0788ab1c3d70431e336f618f39e7eaf046739c

Rules SHA-256: cfc12c1eaa6ea3870e050009e8c7dbe59087ec4933671298f6377e1def624b20

Legacy pre-launch subtotal matches: true.

Raw inventories and method cross-check: source.json, port.json, summary.json.
