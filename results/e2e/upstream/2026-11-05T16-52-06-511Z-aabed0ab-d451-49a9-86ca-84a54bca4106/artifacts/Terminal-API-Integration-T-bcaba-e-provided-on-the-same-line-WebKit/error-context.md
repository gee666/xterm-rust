# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: Terminal.test.ts >> API Integration Tests >> registerLinkProvider >> should work when multiple links are provided on the same line
- Location: test\playwright\Terminal.test.ts:983:9

# Error details

```
Error: page.evaluate: TypeError: window.term.registerLinkProvider is not a function. (In 'window.term.registerLinkProvider({
          provideLinks: (position, cb) => {
            window.calls.push('provide ' + position);
            if (position === 1) {
              cb([{
                range: { start: { x: 1, y: 1 }, end: { x: 3, y: 1 } },
                text: '',
                activate: () => window.calls.push('activate'),
                hover: () => window.calls.push('hover 1-3'),
                leave: () => window.calls.push('leave 1-3')
              }, {
                range: { start: { x: 5, y: 1 }, end: { x: 7, y: 1 } },
                text: '',
                activate: () => window.calls.push('activate'),
                hover: () => window.calls.push('hover 5-7'),
                leave: () => window.calls.push('leave 5-7')
              }, {
                range: { start: { x: 9, y: 1 }, end: { x: 11, y: 1 } },
                text: '',
                activate: () => window.calls.push('activate'),
                hover: () => window.calls.push('hover 9-11'),
                leave: () => window.calls.push('leave 9-11')
              }]);
            }
          }
        })', 'window.term.registerLinkProvider' is undefined)
```

# Page snapshot

```yaml
- generic [ref=e5]:
  - generic [ref=e6]:
    - textbox "Terminal input" [active] [ref=e7]
    - text: WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW
  - generic:
    - generic:
      - generic: foo bar baz
```

# Test source

```ts
  891  |       await moveMouseCell(dims, 7, 1);
  892  |       await timeout(100);
  893  |       await moveMouseCell(dims, 8, 1);
  894  |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'match', 'hover', 'leave', 'hover', 'leave']);
  895  |       await ctx.page.evaluate(`window.disposable.dispose()`);
  896  |     });
  897  | 
  898  |     test('should work fine when hover and leave callbacks are not provided', async () => {
  899  |       await openTerminal(ctx);
  900  |       // Focus the terminal as the cursor will show and trigger a rerender, which can clear the
  901  |       // active link
  902  |       await ctx.page.evaluate('window.term.focus()');
  903  |       await ctx.proxy.write('foo bar baz');
  904  |       // Wait for renderer to catch up as links are cleared on render
  905  |       await pollFor(ctx.page, `document.querySelector('.xterm-rows').textContent`, 'foo bar baz ');
  906  |       await ctx.page.evaluate(`
  907  |         window.calls = [];
  908  |         window.disposable = window.term.registerLinkProvider({
  909  |           provideLinks: (position, cb) => {
  910  |             window.calls.push('provide ' + position);
  911  |             if (position === 1) {
  912  |               window.calls.push('match 1');
  913  |               cb([{
  914  |                 range: { start: { x: 5, y: 1 }, end: { x: 7, y: 1 } },
  915  |                 text: 'bar',
  916  |                 activate: () => window.calls.push('activate')
  917  |               }]);
  918  |             } else if (position === 2) {
  919  |               window.calls.push('match 2');
  920  |               cb([{
  921  |                 range: { start: { x: 5, y: 2 }, end: { x: 7, y: 2 } },
  922  |                 text: 'bar',
  923  |                 activate: () => window.calls.push('activate')
  924  |               }]);
  925  |             }
  926  |           }
  927  |         });
  928  |       `);
  929  |       const dims = await getDimensions();
  930  |       await moveMouseCell(dims, 5, 1);
  931  |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'match 1']);
  932  |       await moveMouseCell(dims, 4, 2);
  933  |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'match 1', 'provide 2', 'match 2']);
  934  |       await moveMouseCell(dims, 7, 1);
  935  |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'match 1', 'provide 2', 'match 2', 'provide 1', 'match 1']);
  936  |       await moveMouseCell(dims, 6, 2);
  937  |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'match 1', 'provide 2', 'match 2', 'provide 1', 'match 1', 'provide 2', 'match 2']);
  938  |       await ctx.page.evaluate(`window.disposable.dispose()`);
  939  |     });
  940  | 
  941  |     test('should fire activate events when clicking the link', async () => {
  942  |       await openTerminal(ctx);
  943  |       // Focus the terminal as the cursor will show and trigger a rerender, which can clear the
  944  |       // active link
  945  |       await ctx.page.evaluate('window.term.focus()');
  946  |       await ctx.proxy.write('a b c');
  947  |       // Wait for renderer to catch up as links are cleared on render
  948  |       await pollFor(ctx.page, `document.querySelector('.xterm-rows').textContent`, 'a b c ');
  949  |       await ctx.page.evaluate(`
  950  |         window.calls = [];
  951  |         window.disposable = window.term.registerLinkProvider({
  952  |           provideLinks: (y, cb) => {
  953  |             window.calls.push('provide ' + y);
  954  |             cb([{
  955  |               range: { start: { x: 1, y }, end: { x: 80, y } },
  956  |               text: window.term.buffer.active.getLine(y - 1).translateToString(),
  957  |               activate: (_, text) => window.calls.push('activate ' + y),
  958  |               hover: () => window.calls.push('hover ' + y),
  959  |               leave: () => window.calls.push('leave ' + y)
  960  |             }]);
  961  |           }
  962  |         });
  963  |       `);
  964  |       const dims = await getDimensions();
  965  |       await moveMouseCell(dims, 3, 1);
  966  |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'hover 1']);
  967  |       await ctx.page.mouse.down();
  968  |       await ctx.page.mouse.up();
  969  |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'hover 1', 'activate 1']);
  970  |       await moveMouseCell(dims, 1, 2);
  971  |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'hover 1', 'activate 1', 'leave 1', 'provide 2', 'hover 2']);
  972  |       await ctx.page.mouse.down();
  973  |       await ctx.page.mouse.up();
  974  |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'hover 1', 'activate 1', 'leave 1', 'provide 2', 'hover 2', 'activate 2']);
  975  |       await moveMouseCell(dims, 5, 1);
  976  |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'hover 1', 'activate 1', 'leave 1', 'provide 2', 'hover 2', 'activate 2', 'leave 2', 'provide 1', 'hover 1']);
  977  |       await ctx.page.mouse.down();
  978  |       await ctx.page.mouse.up();
  979  |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'hover 1', 'activate 1', 'leave 1', 'provide 2', 'hover 2', 'activate 2', 'leave 2', 'provide 1', 'hover 1', 'activate 1']);
  980  |       await ctx.page.evaluate(`window.disposable.dispose()`);
  981  |     });
  982  | 
  983  |     test('should work when multiple links are provided on the same line', async () => {
  984  |       await openTerminal(ctx);
  985  |       // Focus the terminal as the cursor will show and trigger a rerender, which can clear the
  986  |       // active link
  987  |       await ctx.page.evaluate('window.term.focus()');
  988  |       await ctx.proxy.write('foo bar baz');
  989  |       // Wait for renderer to catch up as links are cleared on render
  990  |       await pollFor(ctx.page, `document.querySelector('.xterm-rows').textContent`, 'foo bar baz ');
> 991  |       await ctx.page.evaluate(`
       |                      ^ Error: page.evaluate: TypeError: window.term.registerLinkProvider is not a function. (In 'window.term.registerLinkProvider({
  992  |         window.calls = [];
  993  |         window.disposable = window.term.registerLinkProvider({
  994  |           provideLinks: (position, cb) => {
  995  |             window.calls.push('provide ' + position);
  996  |             if (position === 1) {
  997  |               cb([{
  998  |                 range: { start: { x: 1, y: 1 }, end: { x: 3, y: 1 } },
  999  |                 text: '',
  1000 |                 activate: () => window.calls.push('activate'),
  1001 |                 hover: () => window.calls.push('hover 1-3'),
  1002 |                 leave: () => window.calls.push('leave 1-3')
  1003 |               }, {
  1004 |                 range: { start: { x: 5, y: 1 }, end: { x: 7, y: 1 } },
  1005 |                 text: '',
  1006 |                 activate: () => window.calls.push('activate'),
  1007 |                 hover: () => window.calls.push('hover 5-7'),
  1008 |                 leave: () => window.calls.push('leave 5-7')
  1009 |               }, {
  1010 |                 range: { start: { x: 9, y: 1 }, end: { x: 11, y: 1 } },
  1011 |                 text: '',
  1012 |                 activate: () => window.calls.push('activate'),
  1013 |                 hover: () => window.calls.push('hover 9-11'),
  1014 |                 leave: () => window.calls.push('leave 9-11')
  1015 |               }]);
  1016 |             }
  1017 |           }
  1018 |         });
  1019 |       `);
  1020 |       const dims = await getDimensions();
  1021 |       await moveMouseCell(dims, 2, 1);
  1022 |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'hover 1-3']);
  1023 |       await moveMouseCell(dims, 6, 1);
  1024 |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'hover 1-3', 'leave 1-3', 'hover 5-7']);
  1025 |       await moveMouseCell(dims, 6, 2);
  1026 |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'hover 1-3', 'leave 1-3', 'hover 5-7', 'leave 5-7', 'provide 2']);
  1027 |       await moveMouseCell(dims, 10, 1);
  1028 |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'hover 1-3', 'leave 1-3', 'hover 5-7', 'leave 5-7', 'provide 2', 'provide 1', 'hover 9-11']);
  1029 |       await ctx.page.evaluate(`window.disposable.dispose()`);
  1030 |     });
  1031 | 
  1032 |     test('should dispose links when hovering away', async () => {
  1033 |       await openTerminal(ctx);
  1034 |       // Focus the terminal as the cursor will show and trigger a rerender, which can clear the
  1035 |       // active link
  1036 |       await ctx.page.evaluate('window.term.focus()');
  1037 |       await ctx.proxy.write('foo bar baz');
  1038 |       // Wait for renderer to catch up as links are cleared on render
  1039 |       await pollFor(ctx.page, `document.querySelector('.xterm-rows').textContent`, 'foo bar baz ');
  1040 |       await ctx.page.evaluate(`
  1041 |         window.calls = [];
  1042 |         window.disposable = window.term.registerLinkProvider({
  1043 |           provideLinks: (position, cb) => {
  1044 |             window.calls.push('provide ' + position);
  1045 |             if (position === 1) {
  1046 |               cb([{
  1047 |                 range: { start: { x: 1, y: 1 }, end: { x: 3, y: 1 } },
  1048 |                 text: '',
  1049 |                 activate: () => window.calls.push('activate'),
  1050 |                 dispose: () => window.calls.push('dispose 1-3'),
  1051 |                 hover: () => window.calls.push('hover 1-3'),
  1052 |                 leave: () => window.calls.push('leave 1-3')
  1053 |               }, {
  1054 |                 range: { start: { x: 5, y: 1 }, end: { x: 7, y: 1 } },
  1055 |                 text: '',
  1056 |                 activate: () => window.calls.push('activate'),
  1057 |                 dispose: () => window.calls.push('dispose 5-7'),
  1058 |                 hover: () => window.calls.push('hover 5-7'),
  1059 |                 leave: () => window.calls.push('leave 5-7')
  1060 |               }, {
  1061 |                 range: { start: { x: 9, y: 1 }, end: { x: 11, y: 1 } },
  1062 |                 text: '',
  1063 |                 activate: () => window.calls.push('activate'),
  1064 |                 dispose: () => window.calls.push('dispose 9-11'),
  1065 |                 hover: () => window.calls.push('hover 9-11'),
  1066 |                 leave: () => window.calls.push('leave 9-11')
  1067 |               }]);
  1068 |             }
  1069 |           }
  1070 |         });
  1071 |       `);
  1072 |       const dims = await getDimensions();
  1073 |       await moveMouseCell(dims, 2, 1);
  1074 |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'hover 1-3']);
  1075 |       await moveMouseCell(dims, 6, 1);
  1076 |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'hover 1-3', 'leave 1-3', 'hover 5-7']);
  1077 |       await moveMouseCell(dims, 6, 2);
  1078 |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'hover 1-3', 'leave 1-3', 'hover 5-7', 'leave 5-7', 'dispose 1-3', 'dispose 5-7', 'dispose 9-11', 'provide 2']);
  1079 |       await moveMouseCell(dims, 10, 1);
  1080 |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'hover 1-3', 'leave 1-3', 'hover 5-7', 'leave 5-7', 'dispose 1-3', 'dispose 5-7', 'dispose 9-11', 'provide 2', 'provide 1', 'hover 9-11']);
  1081 |       await moveMouseCell(dims, 10, 2);
  1082 |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'hover 1-3', 'leave 1-3', 'hover 5-7', 'leave 5-7', 'dispose 1-3', 'dispose 5-7', 'dispose 9-11', 'provide 2', 'provide 1', 'hover 9-11', 'leave 9-11', 'dispose 1-3', 'dispose 5-7', 'dispose 9-11', 'provide 2']);
  1083 |       await ctx.page.evaluate(`window.disposable.dispose()`);
  1084 |     });
  1085 |   });
  1086 | });
  1087 | 
  1088 | interface IDimensions {
  1089 |   top: number;
  1090 |   left: number;
  1091 |   renderDimensions: IRenderDimensions;
```