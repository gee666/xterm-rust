import { readdir } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import path from 'node:path';
import { regularFile } from '../acceptance-validation.mjs';

// Separate source-family owners can add cases without editing this shared route.
export async function runCase(context) {
  const directory = path.join(context.root, 'scripts/acceptance-adapters/terminal');
  const files = (await readdir(directory)).filter(file => file.endsWith('.cases.mjs')).sort();
  const handlers = new Map();
  for (const file of files) {
    const relative = `scripts/acceptance-adapters/terminal/${file}`;
    const module = await import(pathToFileURL(await regularFile(context.root, relative)).href);
    if (!Array.isArray(module.caseIds) || module.caseIds.length === 0 || typeof module.runCase !== 'function') {
      throw new Error(`invalid terminal case module ${file}`);
    }
    for (const id of module.caseIds) {
      if (!/^vt-\d+$/.test(id) || handlers.has(id)) throw new Error(`invalid or duplicate terminal case ${id}`);
      handlers.set(id, module.runCase);
    }
  }
  const handler = handlers.get(context.caseId);
  if (!handler) throw new Error(`no original/Rust terminal adapter registered for ${context.caseId}`);
  return handler(context);
}
