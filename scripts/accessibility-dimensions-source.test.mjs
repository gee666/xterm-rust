// MIT. Source-method probes with controlled field hosts, not DOM/browser proof.
import test from 'node:test';
import assert from 'node:assert/strict';
import {sourceDimensionsCase} from '../crates/xterm-web/tests/accessibility_support/dimensions.js';
class Row {
  constructor(width) {this._style={height:'16px',transform:''};this.width=width;}
  get style() {return this._style;}
  getBoundingClientRect() {return {width:this.width};}
}
class Container {
  constructor(parent) {
    this.parentNode=parent;
    this._style={width:'64px',fontSize:'12px'};
    this.rows=[40,8,32].map(width=>new Row(width));
  }
  get style() {return this._style;}
  querySelectorAll() {return this.rows;}
  cloneNode() {return new Container(this.parentNode);}
  remove() {
    const index=this.parentNode.children.indexOf(this);
    assert.ok(index>=0);
    this.parentNode.children.splice(index,1);
  }
}
for(const [mode,width,fontSize,heights,failure] of [
  ['plain','64px','12px',['16px','16px','16px'],'none'],
  ['style-get','128px','18px',['24px','24px','24px'],'none'],
  ['width-mutate','64px','12px',['24px','24px','24px'],'none'],
  ['height-mutate','64px','12px',['16px','24px','24px'],'none'],
  ['row-stop','64px','12px',['20px','16px','16px'],'none'],
  ['style-throw','64px','12px',['16px','16px','16px'],'marker'],
  ['width-throw','64px','12px',['16px','16px','16px'],'marker'],
  ['row-throw','64px','12px',['16px','16px','16px'],'marker'],
]) {
  test(`unchanged accessibility dimension methods: ${mode}`,()=>{
    const parent={children:[],appendChild(child){this.children.push(child);}};
    const result=sourceDimensionsCase(new Container(parent),mode);
    assert.deepEqual({width:result.width,fontSize:result.fontSize,heights:result.heights,failure:result.failure},
      {width,fontSize,heights,failure});
    assert.equal(parent.children.length,0,'probe removes only its own clone');
    console.log(JSON.stringify({mode,...result,scope:'source methods on controlled fields, not real DOM'}));
  });
}
