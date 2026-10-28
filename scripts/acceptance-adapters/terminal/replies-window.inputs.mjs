// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// Case inputs only. Execution and independent captures belong to the shared runner.
export const caseIds=['vt-32','vt-33','vt-35','vt-36','vt-37','vt-38','vt-39','vt-42','vt-43'];
const hex=text=>Array.from({length:text.length},(_,i)=>text.charCodeAt(i).toString(16)).join(',');
const w=text=>`W\t${hex(text)}`;
const option=(key,value)=>`O\t${key}\t${typeof value==='boolean'?'b':typeof value==='number'?'n':typeof value==='string'?'s':'m'}\t${typeof value==='boolean'?+value:typeof value==='string'?hex(value):typeof value==='number'?value:Object.entries(value).map(([name,flag])=>`${name}=${+flag}`).join(',')}`;
const scenario=(name,ops,cols=8,rows=5,scrollback=6)=>({name,input:[`N\t${cols}\t${rows}\t${scrollback}`,...ops].join('\n')+'\n'});
const esc='\x1b';
export function scenarios(caseId){
    if(!caseIds.includes(caseId))throw new Error(`unsupported replies/window case ${caseId}`);
    switch(caseId){
        case 'vt-32':return [['escape',`${esc}7`,`${esc}8`],['csi',`${esc}[s`,`${esc}[u`]].map(([name,save,restore])=>scenario(name,[
            w(restore),w('0123456789\r\nmore'),w(`${esc}[2;4r${esc}[?6h${esc}[?7l${esc}[2;3H${esc}[1;31;44m`),
            w(save),w(`${esc}[?6l${esc}[?7h${esc}[1;1H${esc}[0m`),w(restore),'R\t5\t3',w(restore),
            w(`${esc}[?1049h`),w(`${esc}[2;2H${esc}[32m`),w(save),w(`${esc}[1;1H${esc}[0m`),w(restore),w(`${esc}[?1049l`),w(restore),
        ]));
        case 'vt-33':return ['soft','hard','direct','pending-csi','pending-osc'].map(kind=>scenario(kind,[
            w('one\r\ntwo\r\nthree\r\nfour\r\nfive\r\nsix'),w(`${esc}[?1049hALT`),w(`${esc}[2;4r${esc}[?6h${esc}[4h${esc}[?25l${esc}[?7l${esc}[1;32;44m${esc}7`),
            ...(kind==='soft'?[w(`${esc}[!p`)]:kind==='hard'?[w(`${esc}c`)]:kind==='direct'?['Q']:kind==='pending-csi'?[w(`${esc}[3`),'Q',w('1mX')]:[w(`${esc}]2;pending`),'Q',w(`tail\x07`)]),
            w('Z'),w(`${esc}8`),w(`${esc}[?1049l`),w(`${esc}[?6$p${esc}[4$p${esc}[?7$p`),
        ]));
        case 'vt-35':return [false,true].flatMap(origin=>[false,true].map(disabled=>scenario(`origin-${origin}-stdin-${disabled}`,[
            option('disableStdin',disabled),w(`${esc}[2;4r`),w(`${esc}[?6${origin?'h':'l'}`),w(`${esc}[2;3H`),
            ...[5,6,0,1,15,25,26,53,999].flatMap(request=>[w(`${esc}[${request}n`),w(`${esc}[?${request}n`)]),
            w(`${esc}[1;1H12345678`),w(`${esc}[6n${esc}[?6n${esc}[5n`),
        ])));
        case 'vt-36':return ['xterm','xterm-256color','rxvt-unicode','screen','linux','unknown',''].map(name=>scenario(`term-${name}`,[
            option('termName',name),...['c','0c','1c','>c','>0c','>1c','>q','>0q','>1q'].map(sequence=>w(`${esc}[${sequence}`)),
        ]));
        case 'vt-37':return ['block','underline','bar'].flatMap(style=>[false,true].map(blink=>scenario(`style-${style}-blink-${blink}`,[
            option('cursorStyle',style),option('cursorBlink',blink),w(`${esc}[2;4r${esc}[1;31;44m${esc}[1\"q${esc}[5 q`),
            ...['m','r',' q','"q','"p','','q','mX','unknown'].map(key=>w(`${esc}P$q${key}${esc}\\`)),
            w(`${esc}[2\"q`),w(`${esc}P$q"q${esc}\\`),option('disableStdin',true),w(`${esc}P$qm${esc}\\`),
        ])));
        case 'vt-38':return [0,1,2].map(selector=>scenario(`stack-selector-${selector}`,[
            option('windowOptions',{pushTitle:true,popTitle:true}),w(`${esc}[23;${selector}t`),
            ...Array.from({length:12},(_,i)=>[w(`${esc}]0;title-${i}-界🙂\x07`),w(`${esc}]1;icon-${i}\x07`),w(`${esc}[22;${selector}t`)]).flat(),
            w(`${esc}]2;after-push\x07`),w(`${esc}]1;after-icon\x07`),
            ...Array.from({length:14},()=>w(`${esc}[23;${selector}t`)),w(`${esc}]0;\ud800\x07`),w(`${esc}[22;${selector}t`),w(`${esc}]0;\x07`),w(`${esc}[23;${selector}t`),
        ]));
        case 'vt-39':{
            const operations=[[1,'restoreWin'],[2,'minimizeWin'],[3,'setWinPosition'],[4,'setWinSizePixels'],[5,'raiseWin'],[6,'lowerWin'],[7,'refreshWin'],[8,'setWinSizeChars'],[9,'maximizeWin'],[10,'fullscreenWin'],[11,'getWinState'],[13,'getWinPosition'],[14,'getWinSizePixels'],[15,'getScreenSizePixels'],[16,'getCellSizePixels'],[18,'getWinSizeChars'],[19,'getScreenSizeChars'],[20,'getIconTitle'],[21,'getWinTitle'],[22,'pushTitle'],[23,'popTitle'],[24,'setWinLines'],[25,'setWinLines']];
            return operations.map(([code,permission])=>scenario(`permission-${permission}-${code}`,[
                w(`${esc}]0;before\x07`),option('windowOptions',{[permission]:false}),w(`${esc}[${code}t`),
                option('windowOptions',{[permission]:true}),...[`${code}t`,`${code};0t`,`${code};1t`,`${code};2t`,`${code};999t`].map(params=>w(`${esc}[${params}`)),
                option('windowOptions',{[permission]:false}),w(`${esc}[${code}t`),w(`${esc}[0t${esc}[12t${esc}[17t`),
                ...(permission==='setWinLines'?[option('windowOptions',{setWinLines:true}),w(`${esc}[?3h${esc}[?3l`)]:[]),
            ]));
        }
        case 'vt-42':return [false,true].map(origin=>scenario(`alignment-origin-${origin}`,[
            w('one\r\ntwo\r\nthree\r\nfour\r\nfive\r\nsix'),w(`${esc}[2;4r${esc}[?6${origin?'h':'l'}${esc}[1;31;44;4:3;58:2::1:2:3m${esc}[2;3H`),
            w(`${esc}#8`),w(`${esc}[6n${esc}P$qr${esc}\\`),w(`${esc}[?1049hALT${esc}#8${esc}[?1049l`),
        ]));
        case 'vt-43':return ['block','underline','bar'].flatMap(style=>[false,true].map(blink=>scenario(`cursor-${style}-${blink}`,[
            option('cursorStyle',style),option('cursorBlink',blink),...['','0','1','2','3','4','5','6','7','8','255'].flatMap(parameter=>[w(`${esc}[${parameter} q`),w(`${esc}P$q q${esc}\\`)]),
        ])));
    }
}
