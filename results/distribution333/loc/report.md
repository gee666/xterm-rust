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
| production-library | 514 | 71153 | 68867 | 2532787 |
| addons | 113 | 15303 | 14774 | 541811 |
| bindings-glue | 25 | 2899 | 2789 | 109372 |
| generated-code-data | 106 | 21423 | 20976 | 968418 |
| mixed-code-data | 0 | 0 | 0 | 0 |
| mixed-code-tests | 53 | 9689 | 9311 | 341618 |
| tests | 935 | 127368 | 124519 | 4714795 |
| test-data | 253 | 520924 | 520827 | 213511509 |
| demos | 11 | 983 | 959 | 42283 |
| tooling | 445 | 39179 | 38803 | 2451199 |
| configuration | 14 | 928 | 897 | 413782 |
| total | 2469 | 809849 | 802722 | 225627574 |

Original inventory SHA-256: 319645846cc7e05738b9283d15a6497ec88bd2c9cb723e25c6986122ca533ece

Port inventory SHA-256: 22553d5d705664cf91b60f3863d5618977f38a81cb25292cbbc672e399165382

Rules SHA-256: cfc12c1eaa6ea3870e050009e8c7dbe59087ec4933671298f6377e1def624b20

Legacy pre-launch subtotal matches: true.

Raw inventories and method cross-check: source.json, port.json, summary.json.
