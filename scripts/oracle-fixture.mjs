import { readFile, writeFile } from 'node:fs/promises';
import { strict as assert } from 'node:assert';

/** Verification never replaces a fixture; regeneration requires an explicit flag. */
export async function verifyOrGenerate(file, expected, generate) {
  if (generate) {
    await writeFile(file, expected);
    return;
  }
  const actual = (await readFile(file, 'utf8')).replace(/\r\n/g, '\n');
  assert.equal(actual, expected, `fixture drift: ${file}`);
}
