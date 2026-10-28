# Ligature font fixtures

The four binary fonts in `crates/xterm-addons/tests/fixtures/ligature-fonts/` are unchanged copies of the supplied `source_repo/addons/addon-ligatures/fonts/` files. They are test data, not MIT-licensed Rust source. Their original embedded copyright, names, trademarks and licensing metadata remain intact. Extracted metadata is retained in `results/font-addons/font-fixture-notices.jsonl`.

- FiraCode-Regular.otf, Fira Code Regular version1.205. Copyright ©2015 Nikita Prokopov. Fira Mono is a trademark of The Mozilla Corporation. Embedded licence says Open Font License1.1 or later. SHA256 `baa4dc8673dba72061481507e8480b09c1cbf9a7884c9989cf206c56bff388fe`.
- iosevka-regular.ttf, Iosevka Regular1.14.1 Nagisa. Copyright2015-2017 Belleve Invis. Embedded licence says SIL Open Font Licence1.1. SHA256 `806642aae15244a2e7b706f135b4da5231302b5612205e1a6592952cd2d8bd36`.
- Monoid-Regular.ttf, Monoid Regular0.61. Copyright2016 Andreas Larsen. Embedded licence offers MIT or OFL. These fixture copies select MIT, retained in `licenses/Monoid-MIT.txt`. SHA256 `fd119e732472cc35803480668f316160b0cfa4d5217e7c995dbe91bd9cf19706`.
- UbuntuMono-Regular.ttf, Ubuntu Mono Regular0.80. Copyright2011 Canonical Ltd. Ubuntu and Canonical are registered trademarks of Canonical Ltd. Font by Dalton Maag Ltd. Embedded metadata specifies Ubuntu Font Licence1.0. SHA256 `b35dd9d2131d5d83a9b87fe9ad22c6288fa3d17688d43302c14da29812417d63`.

`Monoid-Regular.woff` is a derived WOFF1 container generated from the unchanged MIT-selected Monoid fixture by `scripts/woff-font-oracle.mjs`. Font table and glyph bytes remain unchanged; zlib compresses tables only for container storage. Its exact SHA256 and source hash are in `results/font-addons/woff-oracle.json`.

Complete licence bodies are in `licenses/OFL-1.1.txt` and `licenses/Ubuntu-font-1.0.txt`. They were retrieved unmodified from the SPDX licence-list-data text collection at:

- https://raw.githubusercontent.com/spdx/license-list-data/main/text/OFL-1.1.txt
- https://raw.githubusercontent.com/spdx/license-list-data/main/text/Ubuntu-font-1.0.txt

The font licence terms apply only to the respective font files. They do not change the Rust code's MIT licence. The fonts are bundled unmodified with the terminal port's tests, not sold as standalone fonts. No endorsement by their authors or trademark holders is claimed.
