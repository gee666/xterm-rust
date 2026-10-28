// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
import assert from 'node:assert/strict';
import {runTrace} from '../../full-trace-runner.mjs';
import {caseIds,scenarios} from './replies-window.inputs.mjs';
export {caseIds};
function integer(text){const value=Number(text);assert.ok(Number.isSafeInteger(value),`invalid trace integer ${text}`);return value;}
function string(text){
    if(text==='')return '';
    return text.split(',').map(unit=>{assert.match(unit,/^[0-9a-f]{1,4}$/i);return String.fromCharCode(Number.parseInt(unit,16));}).join('');
}
function option(kind,text){
    if(kind==='b'){assert.match(text,/^[01]$/);return text==='1';}
    if(kind==='n')return Number(text);
    if(kind==='s')return string(text);
    assert.equal(kind,'m');
    return Object.fromEntries(text.split(',').filter(Boolean).map(entry=>{const parts=entry.split('=');assert.equal(parts.length,2);assert.match(parts[1],/^[01]$/);return [parts[0],parts[1]==='1'];}));
}
export function tracesForCase(caseId){
    return scenarios(caseId).map(({name,input})=>{
        // Split only newlines so an empty final UTF-16 string remains an argument.
        const raw=input.split('\n');if(raw.at(-1)==='')raw.pop();
        const fields=raw.map(line=>line.split('\t'));
        const first=fields.shift();assert.equal(first[0],'N');assert.equal(first.length,4);
        const ops=fields.map(parts=>{
            const [kind,...args]=parts;
            if(kind==='W'){assert.equal(args.length,1);return ['W',string(args[0])];}
            if(kind==='R'){assert.equal(args.length,2);return ['R',...args.map(integer)];}
            if(kind==='S'){assert.equal(args.length,1);return ['S',integer(args[0])];}
            if(kind==='O'){assert.equal(args.length,3);return ['O',args[0],option(args[1],args[2])];}
            assert.ok(kind==='C'||kind==='Q');assert.equal(args.length,0);return [kind];
        });
        assert.ok(ops.length>0);
        return {name,cols:integer(first[1]),rows:integer(first[2]),scrollback:integer(first[3]),ops};
    });
}
export async function runCase(context){
    assert.ok(caseIds.includes(context.caseId),`unsupported terminal reply/window case ${context.caseId}`);
    const result=await runTrace(context,tracesForCase(context.caseId));
    return {caseId:context.caseId,...result};
}
