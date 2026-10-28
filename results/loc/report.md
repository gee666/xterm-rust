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
| production-library | 300 | 41635 | 39855 | 1472946 |
| addons | 60 | 8104 | 7722 | 289290 |
| bindings-glue | 3 | 185 | 169 | 6470 |
| generated-code-data | 14 | 8759 | 8705 | 230164 |
| mixed-code-data | 0 | 0 | 0 | 0 |
| mixed-code-tests | 23 | 3758 | 3558 | 133872 |
| tests | 321 | 50338 | 48928 | 1752498 |
| test-data | 112 | 285244 | 285150 | 115323176 |
| demos | 10 | 942 | 920 | 40886 |
| tooling | 189 | 18883 | 18625 | 1116803 |
| configuration | 10 | 773 | 752 | 47099 |
| total | 1042 | 418621 | 414384 | 120413204 |

Original inventory SHA-256: 5fad15603e7f9938ea11ea5889426ddd4e029a9c8e0540d39de2989dcfa858aa

Port inventory SHA-256: f0e2a64551619b3c932cacd1164b8209bb17ec791dad671b2c78eb8af564198b

Rules SHA-256: 19b600906bf4919b08b72a26394f59728986e3bd6d1e2d0055a38c72c7b9a34c

Legacy pre-launch subtotal matches: true.

Raw inventories and method cross-check: source.json, port.json, summary.json.
