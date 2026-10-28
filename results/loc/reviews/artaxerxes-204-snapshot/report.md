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
| production-library | 355 | 50260 | 48354 | 1794722 |
| addons | 86 | 12136 | 11698 | 422626 |
| bindings-glue | 3 | 188 | 172 | 6776 |
| generated-code-data | 16 | 8945 | 8887 | 236835 |
| mixed-code-data | 0 | 0 | 0 | 0 |
| mixed-code-tests | 38 | 6082 | 5788 | 213508 |
| tests | 411 | 63280 | 61611 | 2233782 |
| test-data | 133 | 423895 | 423801 | 133730730 |
| demos | 11 | 983 | 959 | 42283 |
| tooling | 232 | 23121 | 22833 | 1389799 |
| configuration | 10 | 795 | 773 | 49146 |
| total | 1295 | 589685 | 584876 | 140120207 |

Original inventory SHA-256: 319645846cc7e05738b9283d15a6497ec88bd2c9cb723e25c6986122ca533ece

Port inventory SHA-256: 0433ff89ef728019d58b6156130a47c13cd7433f31d81df3b896dcc81d213888

Rules SHA-256: 21bb893f4aabbc51a1caae0d671190bd8ba0e63bcfc84a6dde475164aca78c4c

Legacy pre-launch subtotal matches: true.

Raw inventories and method cross-check: source.json, port.json, summary.json.
