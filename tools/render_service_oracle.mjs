// Runs the actual unchanged source RenderService on bounded clocks and renderer doubles.
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { withRenderHarness } from '../crates/xterm-web/tests/support/render_service_host.mjs';
if (process.argv.slice(2).some(arg => arg !== '--write-fixtures')) throw new Error('Usage: node tools/render_service_oracle.mjs [--write-fixtures]');
const require = createRequire(path.resolve('source_repo/package.json'));
const base = path.resolve('source_repo/out-esbuild');
const { RenderService } = require(path.join(base, 'browser/services/RenderService.js'));
const { Emitter } = require(path.join(base, 'common/Event.js'));
const { OptionsService } = require(path.join(base, 'common/services/OptionsService.js'));
const { MockBufferService, MockCoreService } = require(path.join(base, 'common/TestUtils.test.js'));
const log = withRenderHarness(host => {
  const log = [];
  class Backend {
    constructor(name) {
      this.name = name; this.redraw = new Emitter(); this.onRequestRedraw = this.redraw.event;
      this.failDispose = false;
      this.dimensions = { css: { canvas: { width: 20, height: 30 }, cell: { width: 5, height: 10 } },
        device: { canvas: { width: 20, height: 30 }, cell: { width: 5, height: 10 }, char: { width: 5, height: 10, left: 0, top: 0 } } };
    }
    renderRows(start,end) { log.push(`${this.name}.rows:${start}:${end}`); }
    handleSelectionChanged(start,end,column) { log.push(`${this.name}.selection:${start ?? '-'}:${end ?? '-'}:${column}`); }
    handleResize(cols,rows) { log.push(`${this.name}.resize:${cols}:${rows}`); this.dimensions.css.canvas.width = cols * 5; this.dimensions.css.canvas.height = rows * 10; }
    dispose() { log.push(`${this.name}.dispose`); if (this.failDispose) throw this.failure; this.redraw.dispose(); }
    handleViewportVisibilityChange(visible) { log.push(`${this.name}.visible:${visible}`); }
    clear() { log.push(`${this.name}.clear`); }
    handleDevicePixelRatioChange() { log.push(`${this.name}.dpr`); }
    handleCharSizeChanged() { log.push(`${this.name}.char`); }
    handleBlur() { log.push(`${this.name}.blur`); }
    handleFocus() { log.push(`${this.name}.focus`); }
    handleCursorMove() { log.push(`${this.name}.cursor`); }
  }
  const buffer = new MockBufferService(4,3), core = new MockCoreService();
  const browser = { window: host.window, onDprChange: new Emitter().event, onWindowChange: new Emitter().event };
  const metrics = { width:5,height:10,hasValidSize:true,measure(){ log.push('measure'); },onCharSizeChange:new Emitter().event };
  const decoration = { onDecorationRegistered:new Emitter().event,onDecorationRemoved:new Emitter().event };
  const theme = { onChangeColors:new Emitter().event };
  const service = new RenderService(3, {}, new OptionsService({}), {warn(){}}, metrics, core, decoration, buffer, browser, theme);
  service.onRenderedViewportChange(event => log.push(`viewport:${event.start}:${event.end}`));
  service.onRender(event => log.push(`render:${event.start}:${event.end}`));
  service.onDimensionsChange(event => log.push(`dimensions:${event.css.canvas.width}:${event.css.canvas.height}`));
  const a=new Backend('A'), b=new Backend('B'), c=new Backend('C');
  service.setRenderer(a); host.frame();
  service.addRefreshCallback(time=>log.push(`frame:${time}`));
  service.refreshRows(2,2,false,true);service.refreshRows(1,1,false,false);host.frame();
  a.redraw.fire({start:0,end:0});host.frame();
  service.setRenderer(a);host.frame();
  a.redraw.fire({start:1,end:1,sync:true});
  core.decPrivateModes.synchronizedOutput=true;service.refreshRows(2,2);host.frame();
  core.decPrivateModes.synchronizedOutput=false;service.refreshRows(1,1);core.decPrivateModes.synchronizedOutput=true;host.frame();
  core.decPrivateModes.synchronizedOutput=false;service.refreshRows(0,0,true,true);
  log.push(`pending:${JSON.stringify(host.pending())}`);
  host.visibility(false);service.resize(6,4);service.handleResize(6,4);service.resize(7,5);service.handleResize(7,5);service.refreshRows(1,2);host.frame();
  host.ratio(1);host.frame();service.resize(7,5);service.resize(7,5);
  service.handleSelectionChanged([1,2],[3,4],true);service.handleFocus();service.handleBlur();service.handleCursorMove();
  service.refreshRows(-2,99,true,false);service.refreshRows(0.5,1.5,true,false);service.refreshRows(NaN,NaN,true,false);
  service.setRenderer(b);host.frame();
  b.failDispose=true;b.failure=new Error('backend dispose failed');
  try { service.setRenderer(c);throw new Error('missing expected failure'); } catch(error) { if(error!==b.failure)throw error;log.push('dispose-error-same:true'); }
  b.redraw.fire({start:2,end:2,sync:true});b.failDispose=false;
  core.decPrivateModes.synchronizedOutput=true;service.refreshRows(3,3);host.safety();host.frame();
  service.refreshRows(0,0);core.decPrivateModes.synchronizedOutput=true;service.refreshRows(2,2);service.dispose();
  service.setRenderer(c);log.push(`has-renderer:${service.hasRenderer()}`);log.push(`pending:${JSON.stringify(host.pending())}`);
  return log;
});
const source = 'source_repo/src/browser/services/RenderService.ts';
const result = { source, sha256:createHash('sha256').update(fs.readFileSync(source)).digest('hex'), log };
const fixture = 'crates/xterm-web/tests/fixtures/render-service.json';
if (process.argv.includes('--write-fixtures')) fs.writeFileSync(fixture,JSON.stringify(result,null,2)+'\n');
else if (fs.readFileSync(fixture,'utf8').replaceAll('\r\n','\n')!==JSON.stringify(result,null,2)+'\n') throw new Error('Source RenderService fixture drift');
fs.mkdirSync('results/renderer',{recursive:true});
fs.writeFileSync('results/renderer/render-service-original.json',JSON.stringify({source,sha256:result.sha256,entries:log.length,status:'passed source trace',scope:'Controlled RenderService component, not real browser terminal E2E'},null,2)+'\n');
console.log(JSON.stringify({entries:log.length,status:'passed source trace'}));
