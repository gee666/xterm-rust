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
| production-library | 328 | 46116 | 44239 | 1651967 |
| addons | 78 | 10493 | 10087 | 374291 |
| bindings-glue | 3 | 186 | 170 | 6649 |
| generated-code-data | 15 | 8909 | 8852 | 235323 |
| mixed-code-data | 0 | 0 | 0 | 0 |
| mixed-code-tests | 34 | 5507 | 5225 | 194043 |
| tests | 374 | 58119 | 56533 | 2051047 |
| test-data | 122 | 413073 | 412979 | 131234836 |
| demos | 11 | 983 | 959 | 42283 |
| tooling | 211 | 21026 | 20750 | 1253893 |
| configuration | 10 | 795 | 773 | 49146 |
| total | 1186 | 565207 | 560567 | 137093478 |

Original inventory SHA-256: 319645846cc7e05738b9283d15a6497ec88bd2c9cb723e25c6986122ca533ece

Port inventory SHA-256: ac7cd47c1af1a42e4d43400ea6d0127c44d7cc194c607e971584ee30b6ab927c

Rules SHA-256: 21bb893f4aabbc51a1caae0d671190bd8ba0e63bcfc84a6dde475164aca78c4c

Legacy pre-launch subtotal matches: true.

Raw inventories and method cross-check: source.json, port.json, summary.json.
