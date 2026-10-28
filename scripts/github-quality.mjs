#!/usr/bin/env node
// Tokens remain in process memory. Neither credential output nor request headers are logged.
import { spawnSync } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';
async function main() {
const root = fileURLToPath(new URL('../', import.meta.url));
const [action, value] = process.argv.slice(2);
if (!['runs', 'dispatch', 'status', 'artifacts', 'logs'].includes(action)) {
  throw new Error('Usage: node scripts/github-quality.mjs runs|dispatch REF|status RUN_ID|artifacts RUN_ID|logs RUN_ID');
}
if (action !== 'runs' && !value) throw new Error('A ref or run ID is required');
if (['status', 'artifacts', 'logs'].includes(action) && !/^\d+$/.test(value)) throw new Error('Run ID must be numeric');
const env = { ...process.env, GIT_TERMINAL_PROMPT: '0', GCM_INTERACTIVE: 'Never' };
const remote = spawnSync('git', ['remote', 'get-url', 'origin'], { cwd: root, env, encoding: 'utf8', timeout: 30_000 });
if (remote.error || remote.status !== 0) throw new Error('Cannot read origin URL');
const match = remote.stdout.trim().match(/github\.com[:/]([^/]+)\/([^/]+?)(?:\.git)?$/);
if (!match) throw new Error('Origin must be a GitHub repository');
const repository = `${match[1]}/${match[2]}`;
let token = process.env.GH_TOKEN || process.env.GITHUB_TOKEN;
if (!token) {
  const credential = spawnSync('git', ['credential', 'fill'], {
    cwd: root, env, encoding: 'utf8', timeout: 30_000,
    input: `protocol=https\nhost=github.com\npath=${repository}.git\n\n`
  });
  if (credential.error || credential.status !== 0) throw new Error('GitHub credentials unavailable; authenticate Git/gh or provide GH_TOKEN privately');
  token = credential.stdout.match(/^password=(.+)$/m)?.[1]?.trim();
  if (!token) throw new Error('Git credential helper returned no password/token');
}
async function api(path, method = 'GET', body) {
  const response = await fetch(`https://api.github.com/repos/${repository}${path}`, {
    method, signal: AbortSignal.timeout(60_000),
    headers: { Authorization: `Bearer ${token}`, Accept: 'application/vnd.github+json',
      'X-GitHub-Api-Version': '2022-11-28', ...(body ? { 'Content-Type': 'application/json' } : {}) },
    body: body ? JSON.stringify(body) : undefined
  });
  if (!response.ok) throw new Error(`GitHub ${method} ${path}: HTTP ${response.status}`);
  return response;
}
if (action === 'dispatch') {
  await api('/actions/workflows/rust.yml/dispatches', 'POST', { ref: value });
  console.log(JSON.stringify({ repository, ref: value, dispatched: true, note: 'Dispatch is not a runtime pass; retrieve the resulting run.' }, null, 2));
} else if (action === 'runs') {
  const runs = await (await api('/actions/runs?per_page=100')).json();
  console.log(JSON.stringify(runs.workflow_runs.filter(run => run.path === '.github/workflows/rust.yml').map(run => ({ id: run.id, sha: run.head_sha, branch: run.head_branch, status: run.status, conclusion: run.conclusion, url: run.html_url })), null, 2));
} else {
  const directory = join(root, 'results', 'quality', `ci-${value}`);
  await mkdir(directory, { recursive: true });
  const run = await (await api(`/actions/runs/${value}`)).json();
  const jobs = await (await api(`/actions/runs/${value}/jobs?per_page=100`)).json();
  const summary = {
    repository, observedAt: new Date().toISOString(), id: run.id, sha: run.head_sha,
    branch: run.head_branch, status: run.status, conclusion: run.conclusion, url: run.html_url,
    jobs: jobs.jobs.map(job => ({ id: job.id, name: job.name, status: job.status, conclusion: job.conclusion,
      runner: job.runner_name, labels: job.labels, steps: job.steps.map(step => ({ name: step.name, status: step.status, conclusion: step.conclusion })) }))
  };
  await writeFile(join(directory, 'run.json'), JSON.stringify(summary, null, 2) + '\n');
  if (action === 'artifacts') {
    const artifacts = await (await api(`/actions/runs/${value}/artifacts?per_page=100`)).json();
    const saved = [];
    for (const artifact of artifacts.artifacts) {
      if (!artifact.name.startsWith('quality-')) continue;
      if (artifact.expired) throw new Error(`Artifact ${artifact.id} expired`);
      if (artifact.size_in_bytes > 50_000_000) throw new Error(`Artifact ${artifact.id} exceeds 50 MB safety limit`);
      const response = await api(`/actions/artifacts/${artifact.id}/zip`);
      const filename = `${artifact.name.replace(/[^\w.-]/g, '_')}-${artifact.id}.zip`;
      const bytes = new Uint8Array(await response.arrayBuffer());
      if (bytes.length > 50_000_000) throw new Error('Downloaded artifact exceeds 50 MB');
      await writeFile(join(directory, filename), bytes);
      saved.push({ id: artifact.id, name: artifact.name, filename, bytes: bytes.length });
    }
    await writeFile(join(directory, 'artifacts.json'), JSON.stringify(saved, null, 2) + '\n');
    summary.savedArtifacts = saved.length;
  }
  if (action === 'logs') {
    for (const job of jobs.jobs) {
      if (job.status !== 'completed') throw new Error(`Job ${job.id} has not completed; log retrieval deferred`);
      const response = await api(`/actions/jobs/${job.id}/logs`);
      const bytes = new Uint8Array(await response.arrayBuffer());
      if (bytes.length > 10_000_000) throw new Error(`Job ${job.id} log exceeds 10 MB`);
      await writeFile(join(directory, `job-${job.id}.log`), bytes);
    }
    summary.savedJobLogs = jobs.jobs.length;
  }
  console.log(JSON.stringify(summary, null, 2));
}
}
await main().catch(error => {
  console.error(error.message);
  process.exitCode = 1;
});
