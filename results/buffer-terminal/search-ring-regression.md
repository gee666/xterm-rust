# Observed source SearchEngine ring recursion

The generator `scripts/buffer-terminal-oracle.mjs` executes the real original headless Terminal and SearchEngine. It observed 80 RangeError exceptions in 4,005 queries. They are retained as expected exceptional outcomes in `crates/xterm-core/tests/fixtures/buffer-terminal.tsv`, not skipped or converted to no-match. `results/buffer-terminal/oracle.json` records each trace/step/query, class and message.

First observed case is seed `0xbaff0099`, trace `seed-0`, step 11. Initial dimensions are 9 columns, 1 row, scrollback 5. Execute these operations in order, awaiting each write callback:

```text
W ESC[2J
W CR LF next
W 中
W 中
W ESC[2J
W e U+0301
W e U+0301
W CR LF next
W CR LF line
W ESC[?1049hALT
R 9 1
W abc
```

Create a fresh original SearchLineCache over that terminal, then a SearchEngine using that cache. Each query `abc`, `中`, `next`, `line`, or `e\u0301`, starting at row zero and column zero, throws JavaScript `RangeError` with message `Maximum call stack size exceeded` in `_findInLine`.

The active alternate buffer has one row and that row is wrapped. SearchEngine decrements startRow while the returned line is wrapped. BufferApiView forwards negative row reads to the cyclic list without a bounds check, which can keep returning wrapped rows indefinitely. A Rust implementation should return a typed error, not exhaust its native stack, panic, or silently report None. Compatible browser bindings must preserve the JavaScript RangeError class/message. This error outcome is part of the observed source contract even though the original code has a bug.

A likely smaller reproducer is a 2x1 terminal with scrollback zero, write `abc`, then search for `c`. This reduction has NOT been executed yet during the coordinated disk pause. The full observed trace above is the evidence.

The permitted source-only regeneration now retains 73 traces and 862 snapshots, including the extra PA06 cases. It still records the same 80 exceptional search outcomes. Rust execution remains pending a coordinated build slot; source capture alone is not Rust equivalence.
