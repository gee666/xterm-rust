// Retain every literal palette expectation in the original ThemeService suite.
import fs from 'node:fs';
const source = 'source_repo/src/browser/services/ThemeService.test.ts';
const input = fs.readFileSync(source, 'utf8');
const start = input.indexOf("it('should fill 240 colors with expected values'");
if (start < 0) throw new Error('The palette test name changed');
const end = input.indexOf('});', start);
const entries = [...input.slice(start, end).matchAll(/assert\.equal\(themeService\.colors\.ansi\[(\d+)\]\.css, '(#[a-f\d]+)'\);/gi)];
if (entries.length !== 240 || entries.some((entry, index) => Number(entry[1]) !== index + 16)) {
  throw new Error('The original explicit 240-color expectation sequence changed');
}
fs.mkdirSync('crates/xterm-web/tests/fixtures', { recursive: true });
fs.writeFileSync('crates/xterm-web/tests/fixtures/theme-palette.tsv', entries.map(([, index, css]) => `${index}\t${css}`).join('\n') + '\n');
console.log(JSON.stringify({ source, retainedAssertions: entries.length }));
