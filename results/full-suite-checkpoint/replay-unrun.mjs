import fs from 'node:fs';
import { runCommand } from '../../scripts/quality-process.mjs';
const raw=JSON.parse(fs.readFileSync('results/e2e/upstream/2026-11-05T12-01-38-775Z-1456484b-4d16-49e6-a14a-19060168ec63/playwright.json'));
const projects=new Map();
function visit(suites){for(const s of suites){for(const spec of s.specs??[])for(const test of spec.tests??[]){const result=test.results.at(-1);const declared=test.expectedStatus==='skipped'||test.annotations?.some(a=>['skip','fixme'].includes(a.type));if(!result||(result.status==='skipped'&&!declared)){const rows=projects.get(test.projectName)??[]; rows.push({title:spec.title,file:spec.file??s.file});projects.set(test.projectName,rows);}}visit(s.suites??[]);}}
visit(raw.suites??[]);
fs.writeFileSync('results/full-suite-checkpoint/core-unrun-selection.json',JSON.stringify([...projects].map(([project,cases])=>({project,cases})),null,2)+'\n');
for(const[project,rows]of projects){const titles=[...new Set(rows.map(r=>r.title))];const grep='(?:'+titles.map(t=>t.replace(/[.*+?^${}()|[\]\\]/g,'\\$&')).join('|')+')$';const args=['scripts/e2e-upstream.mjs','--suite','core','--no-build','--project',project,'--grep',grep,'--timeout-ms','480000'];const r=await runCommand(process.execPath,args,{cwd:process.cwd(),env:process.env,log:`tmp/core-unrun-${project}.log`,timeoutMs:510000});console.log(project,rows.length,r.status);console.log(fs.readFileSync(`tmp/core-unrun-${project}.log`,'utf8').slice(-1400));}
