// Retains the former strict miniz output-boundary regression, restores exact bytes in finally.
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const file='crates/xterm-addons/src/ligatures/font_cff_header.rs',original=await fs.readFile(file,'utf8');
const hash=x=>createHash('sha256').update(x).digest('hex');
const needle='        header(Reader::new(&prefix[..size]))';
if(original.split(needle).length!==2)throw new Error('Negative-control anchor drift');
const guard=`        let inflated = miniz_oxide::inflate::decompress_to_vec_with_limit(reader.bytes(input, length)?, record.offset32(12)?)
            .map_err(|_| FontError::Compression)?;
        if inflated.len() != record.offset32(12)? { return Err(FontError::Compression); }
`;
function run(name){const result=spawnSync(process.env.CARGO??'cargo',['test','--locked','-j2','-p','xterm-addons','--test','font_cff_header','--','--nocapture'],{encoding:'utf8',timeout:180000,maxBuffer:16*1024*1024,env:{...process.env,CARGO_BUILD_JOBS:'2'}});if(result.error)throw result.error;return fs.writeFile(`results/font-cff-header/${name}.log`,result.stdout+result.stderr).then(()=>result.status);}
let failed,restored;
try {await fs.writeFile(file,original.replace(needle,guard+needle));failed=await run('negative-strict-output');if(failed!==101)throw new Error(`Expected test failure101, got ${failed}`);}
finally {await fs.writeFile(file,original);if(hash(await fs.readFile(file))!==hash(original))throw new Error('Source restoration mismatch');restored=await run('restored-prefix');}
if(restored!==0)throw new Error(`Restored producer failed ${restored}`);
await fs.writeFile('results/font-cff-header/negative.json',JSON.stringify({scope:'Former strict miniz output-size rejection is wrong for short/overflow/missing-stored-payload CFF headers',status:failed,restoredStatus:restored,restoredSha256:hash(original)},null,2)+'\n');console.log('Old strict output boundary fails; exact restored source passes');
