// Count physical/nonblank lines, not comment-stripped SLOC. No dependencies.
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, mkdirSync, writeFileSync } from 'node:fs';
import { dirname, extname, isAbsolute, relative, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const project = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const hash = value => createHash('sha256').update(value).digest('hex');
const slash = value => value.replaceAll('\\', '/');
const empty = () => ({ files: 0, physical: 0, nonblank: 0, bytes: 0 });
const categories = ['production-library', 'addons', 'bindings-glue', 'generated-code-data', 'mixed-code-data',
  'tests', 'test-data', 'demos', 'tooling', 'configuration'];
const decoder = new TextDecoder('utf-8', { fatal: true });

export function lineCounts(text) {
  // LF and CRLF delimit lines. A final delimiter does not create a phantom line.
  const lines = text.length ? text.split(/\r?\n/) : [];
  if (text.endsWith('\n')) lines.pop();
  return { physical: lines.length, nonblank: lines.filter(line => line.trim()).length };
}

function add(total, row) {
  total.files++;
  total.physical += row.physical;
  total.nonblank += row.nonblank;
  total.bytes += row.bytes;
}

export function purpose(path, tree) {
  if (/(^|\/)(tests?|fixtures?)(\/|$)|\.(test|spec)\.[^.]+$|(?:_tests|_test)\.rs$|_upstream_cases\.tsv$|(^|\/)TestUtils\./.test(path)) return 'tests';
  if (/(^|\/)(demo|demos)(\/|$)/.test(path)) return 'demos';
  if (/(^|\/)(benchmark|benchmarks|benches|bin|scripts|tools)(\/|$)|(?:^|\/)(webpack|eslint)\.config/.test(path)) return 'tooling';
  // Differential adapters are verification tooling, not user demonstrations.
  if (/\/examples\/(?:.*oracle|.*trace)\.rs$/.test(path)) return 'tooling';
  if (/(^|\/)(examples)(\/|$)/.test(path)) return 'demos';
  if (/\.d\.ts$|(^|\/)(typings|bindings|glue)(\/|$)|^crates\/[^/]*(?:wasm|binding)[^/]*\//.test(path)) return 'bindings-glue';
  if (tree === 'source' && /^addons\//.test(path)) return 'addons';
  if (tree === 'port' && /^(?:addons\/|crates\/(?:(?:xterm-)?addon-|xterm-addons\/))/.test(path)) return 'addons';
  if (/^(?:src|css|headless|crates)\//.test(path)) return 'production-library';
  return 'tooling';
}

function legacyPurpose(path) {
  return /(^|\/)(test|tests)(\/|$)|\.(test|spec)\.[^.]+$|(^|\/)TestUtils\./.test(path)
    ? 'tests' : /^(src|addons|typings|headless|css)\//.test(path) ? 'library' : 'demo-and-tooling';
}

export function inventory(root, tree, rules, excludedRoots = []) {
  const rows = [];
  const excluded = [];
  const unknownProduction = [];
  const totals = empty();
  const byCategory = Object.fromEntries(categories.map(category => [category, empty()]));
  const byPurpose = {};
  const byExtension = {};
  const baselineCompatible = { groups: {}, totals: empty() };
  const binaryAssets = { files: 0, bytes: 0 };
  const mixed = new Set(rules.mixedSourceData[tree]);
  const generatedPaths = new Set(rules.generatedOverrides[tree]);
  function walk(directory) {
    const entries = readdirSync(directory, { withFileTypes: true })
      .sort((a, b) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0);
    for (const entry of entries) {
      const absolute = resolve(directory, entry.name);
      const path = slash(relative(root, absolute));
      const top = !path.includes('/');
      if (excludedRoots.some(excluded => path === excluded || path.startsWith(excluded + '/')) ||
          (entry.isDirectory() && rules.excludedDirectories.includes(entry.name)) ||
          (tree === 'port' && top && (rules.portExcludedRoots.includes(entry.name) || rules.portExcludedFiles.includes(entry.name)))) {
        excluded.push({ path, reason: 'explicit exclusion rule' });
        continue;
      }
      if (entry.isSymbolicLink()) throw new Error(`Symlink requires explicit inventory review: ${path}`);
      if (entry.isDirectory()) { walk(absolute); continue; }
      if (!entry.isFile()) throw new Error(`Unsupported file type: ${path}`);
      const bytes = readFileSync(absolute);
      const extension = extname(path).toLowerCase();
      const role = rules.bindingOverrides[tree].includes(path) ? 'bindings-glue' : purpose(path, tree);
      const source = rules.sourceExtensions.includes(extension) || (extension === '' && /(^|\/)bin\//.test(path));
      const configuration = rules.configurationExtensions.includes(extension);
      const fixture = role === 'tests' && rules.fixtureExtensions.includes(extension);
      const productionData = ['production-library', 'addons', 'bindings-glue'].includes(role) && rules.fixtureExtensions.includes(extension);
      const lock = rules.lockFiles.includes(entry.name);
      const row = { path, bytes: bytes.length, sha256: hash(bytes), extension, purpose: role, counted: false };
      if ((source || configuration || fixture || productionData) && !lock) {
        let text;
        try { text = decoder.decode(bytes); } catch (error) {
          if (source || configuration) throw new Error(`Invalid UTF-8 source/configuration: ${path}`, { cause: error });
        }
        if (text !== undefined) {
          const generated = generatedPaths.has(path) || text.slice(0, 2048).split(/\r?\n/)
            .some(line => /^\s*(?:\/\/|\/\*|\*|#|<!--).*(?:generated by|automatically generated|@generated|do not edit by hand)/i.test(line));
          const category = configuration ? 'configuration' : generated || productionData ? 'generated-code-data' : mixed.has(path) ? 'mixed-code-data' : fixture ? 'test-data' : role;
          Object.assign(row, lineCounts(text), { counted: true, category, generated, mixedSourceData: mixed.has(path) });
          add(totals, row);
          add(byCategory[category], row);
          add(byPurpose[role] ??= empty(), row);
          add(byExtension[extension || '(extensionless)'] ??= empty(), row);
        }
      }
      if (!row.counted) {
        row.reason = lock ? 'dependency lockfile' : 'asset, document or unrecognized source extension';
        binaryAssets.files++;
        binaryAssets.bytes += bytes.length;
        if (['production-library', 'addons', 'bindings-glue'].includes(role) &&
            !lock && !/\.(?:md|png|jpg|jpeg|gif|svg|ico|woff2?|ttf|otf|wasm|map|txt)$/i.test(path) &&
            !/(^|\/)(LICENSE|NOTICE|\.gitignore|\.npmignore|\.gitattributes)$/.test(path)) unknownProduction.push(path);
      }
      if (tree === 'source' && rules.baselineExtensions.includes(extension)) {
        const counts = lineCounts(decoder.decode(bytes));
        const group = `${legacyPurpose(path)} ${extension}`;
        const legacyRow = { ...counts, bytes: bytes.length };
        add(baselineCompatible.totals, legacyRow);
        add(baselineCompatible.groups[group] ??= empty(), legacyRow);
      }
      rows.push(row);
    }
  }
  walk(root);
  return { tree, totals, byCategory, byPurpose, byExtension, baselineCompatible,
    uncountedAssetsAndDocuments: binaryAssets, unknownProduction, excluded,
    inventorySha256: hash(JSON.stringify(rows)), files: rows };
}

export function main(args = process.argv.slice(2)) {
  const options = { source: resolve(project, 'source_repo'), port: project,
    output: resolve(project, 'results/loc'), rules: resolve(project, 'scripts/loc-rules.json') };
  let checkBaseline = false;
  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--check-baseline') { checkBaseline = true; continue; }
    const key = args[i].slice(2);
    if (!args[i].startsWith('--') || !Object.hasOwn(options, key) || !args[i + 1] || args[i + 1].startsWith('--')) throw new Error(`Invalid argument: ${args[i]}`);
    options[key] = resolve(args[++i]);
  }
  const ruleBytes = readFileSync(options.rules);
  const rules = JSON.parse(ruleBytes);
  if ([options.source, options.port].includes(options.output)) throw new Error('Output directory cannot equal a source root');
  const nestedPaths = root => [options.output, options.source]
    .map(path => slash(relative(root, path)))
    .filter(path => path && !path.startsWith('../') && !isAbsolute(path));
  const source = inventory(options.source, 'source', rules, nestedPaths(options.source));
  const port = inventory(options.port, 'port', rules, nestedPaths(options.port));
  if (source.unknownProduction.length || port.unknownProduction.length) {
    throw new Error(`Unclassified production files require a rule: ${JSON.stringify({ source: source.unknownProduction, port: port.unknownProduction })}`);
  }
  const baseline = JSON.parse(readFileSync(resolve(project, 'docs/baseline/source-loc.json'), 'utf8'));
  const expected = { files: baseline.totals.files, physical: baseline.totals.lines,
    nonblank: baseline.totals.nonblank, bytes: baseline.totals.bytes };
  const baselineMatches = JSON.stringify(source.baselineCompatible.totals) === JSON.stringify(expected);
  const summary = {
    schemaVersion: 1, tool: 'xterm-rust count-loc.mjs v1', node: process.version,
    method: 'UTF-8 physical LF/CRLF lines and nonblank lines, including comments; no comment-stripped SLOC',
    status: 'Partial-port working-tree snapshot. Not evidence of equivalent-functionality code reduction.',
    rulesSha256: hash(ruleBytes), source: { totals: source.totals, byCategory: source.byCategory, inventorySha256: source.inventorySha256 },
    port: { totals: port.totals, byCategory: port.byCategory, inventorySha256: port.inventorySha256 },
    baseline: { expected, actual: source.baselineCompatible.totals, matches: baselineMatches }
  };
  mkdirSync(options.output, { recursive: true });
  for (const [name, report] of Object.entries({ source, port, summary })) {
    writeFileSync(resolve(options.output, `${name}.json`), JSON.stringify(report, null, 2) + '\n');
  }
  const table = tree => {
    const rows = Object.entries(tree.byCategory).concat([['total', tree.totals]])
      .map(([category, value]) => `| ${category} | ${value.files} | ${value.physical} | ${value.nonblank} | ${value.bytes} |`);
    return ['| Category | Files | Physical | Nonblank | Bytes |', '| --- | ---: | ---: | ---: | ---: |', ...rows].join('\n');
  };
  const markdown = ['# Partial-port source line snapshot', '', summary.status, '',
    summary.method + '.', '', `Tool: ${summary.tool}, Node ${summary.node}.`, '',
    '## Original', '', table(source), '', '## Rust port working tree', '', table(port), '',
    `Original inventory SHA-256: ${source.inventorySha256}`, '',
    `Port inventory SHA-256: ${port.inventorySha256}`, '',
    `Rules SHA-256: ${summary.rulesSha256}`, '',
    `Legacy pre-launch subtotal matches: ${baselineMatches}.`, '',
    'Raw inventories and method cross-check: source.json, port.json, summary.json.', ''].join('\n');
  writeFileSync(resolve(options.output, 'report.md'), markdown);
  console.log(JSON.stringify(summary, null, 2));
  if (checkBaseline && !baselineMatches) throw new Error('Pre-launch baseline totals differ; inspect source.json before interpreting measurements');
  return summary;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) main();
