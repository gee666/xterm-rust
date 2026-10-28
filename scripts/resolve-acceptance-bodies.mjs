// Export support for the live swarm only. Normal builds never need .pi data.
import { readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { validateRegistry } from './check-acceptance-index.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const filename = path.join(root, 'results/acceptance/criteria-index.json');
const index = JSON.parse(await readFile(filename, 'utf8'));
validateRegistry(index);
let resolved = 0;
for (const criterion of index.criteria) {
  for (const field of ['body', 'verification']) {
    const match = criterion[field].match(/\[full text: ([^\]]+)\]/);
    if (!match) continue;
    const relative = match[1].replaceAll('\\', '/');
    if (!/^\.pi\/swarm\/attachments\/\d+\/work\/[a-f0-9-]+\.md$/.test(relative)) {
      throw new Error(`unsupported attachment reference in criterion ${criterion.id}`);
    }
    criterion[field] = await readFile(path.join(root, relative), 'utf8');
    resolved++;
  }
}
validateRegistry(index);
await writeFile(filename, JSON.stringify(index, null, 2) + '\n');
console.log(JSON.stringify({ criteria: index.criteria.length, resolvedAttachmentBodies: resolved }));
