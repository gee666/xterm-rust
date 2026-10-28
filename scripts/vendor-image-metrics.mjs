// Exact reference fixture copies let native Rust tests run without source_repo.
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import assert from 'node:assert/strict';
const require=createRequire(import.meta.url);
const root=path.resolve(import.meta.dirname,'..');
const reference='source_repo/addons/addon-image';
const destination='crates/xterm-addons/tests/fixtures/image-metrics';
const manifestFile=path.join(root,destination,'manifest.json');
const hash=data=>createHash('sha256').update(data).digest('hex');
const absolute=file=>path.join(root,file);
const basename=file=>typeof file==='string'&&file.length>0&&path.basename(file)===file&&!file.includes('\\');
function bytes(file){assert.ok(fs.lstatSync(file).isFile(),`not a regular fixture: ${file}`);return fs.readFileSync(file);}
if(process.argv.includes('--write')){
 const ts=require('../source_repo/node_modules/typescript');
 const testPath=`${reference}/src/IIPMetrics.test.ts`,source=bytes(absolute(testPath));
 const file=ts.createSourceFile(testPath,source.toString('utf8'),ts.ScriptTarget.Latest,true);
 let initializer;
 for(const statement of file.statements)if(ts.isVariableStatement(statement))for(const declaration of statement.declarationList.declarations)if(declaration.name.getText(file)==='TEST_IMAGES')initializer=declaration.initializer;
 assert.ok(initializer&&ts.isArrayLiteralExpression(initializer),'missing original TEST_IMAGES');
 const names=initializer.elements.map(tuple=>{assert.ok(ts.isArrayLiteralExpression(tuple)&&ts.isStringLiteral(tuple.elements[0]));return tuple.elements[0].text;});
 assert.equal(names.length,38,'upstream fixture inventory changed; review the complete suite');assert.equal(new Set(names).size,names.length);
 fs.mkdirSync(absolute(destination),{recursive:true});
 const files=names.map(name=>{
  assert.ok(basename(name));const sourcePath=`${reference}/fixture/testimages/${name}`;
  const data=bytes(absolute(sourcePath));const targetPath=`${destination}/${name}`;
  if(fs.existsSync(absolute(targetPath)))assert.equal(hash(bytes(absolute(targetPath))),hash(data),`refusing to overwrite changed fixture ${name}`);
  else fs.copyFileSync(absolute(sourcePath),absolute(targetPath));
  return{name,sourcePath,path:targetPath,bytes:data.length,sha256:hash(data)};
 });
 const licenseSource=`${reference}/LICENSE`,licensePath=`${destination}/LICENSE.xterm.txt`;
 fs.copyFileSync(absolute(licenseSource),absolute(licensePath));
 const manifest={schemaVersion:1,provenance:'Exact user-provided source_repo xterm.js6.0.0 reference fixtures. No public Git revision is asserted.',sourceTest:{path:testPath,sha256:hash(source)},license:{sourcePath:licenseSource,path:licensePath,sha256:hash(bytes(absolute(licensePath)))},files};
 fs.writeFileSync(manifestFile,JSON.stringify(manifest,null,2)+'\n');
}
const manifest=JSON.parse(bytes(manifestFile));assert.equal(manifest.schemaVersion,1);assert.equal(manifest.files.length,38);
for(const file of manifest.files){
 assert.ok(basename(file.name));assert.equal(file.path,`${destination}/${file.name}`);assert.equal(file.sourcePath,`${reference}/fixture/testimages/${file.name}`);
 const data=bytes(absolute(file.path));assert.equal(data.length,file.bytes);assert.equal(hash(data),file.sha256);
 if(fs.existsSync(absolute(file.sourcePath)))assert.equal(hash(bytes(absolute(file.sourcePath))),file.sha256,`reference differs: ${file.name}`);
}
assert.equal(manifest.license.path,`${destination}/LICENSE.xterm.txt`);
assert.equal(manifest.license.sourcePath,`${reference}/LICENSE`);
assert.equal(manifest.sourceTest.path,`${reference}/src/IIPMetrics.test.ts`);
assert.equal(hash(bytes(absolute(manifest.license.path))),manifest.license.sha256);
console.log(JSON.stringify({status:'passed',scope:'Fixture bytes/notice only, not image decoding or browser parity',files:manifest.files.length,bytes:manifest.files.reduce((sum,file)=>sum+file.bytes,0),sourceTest:manifest.sourceTest,manifestSha256:hash(bytes(manifestFile))},null,2));
