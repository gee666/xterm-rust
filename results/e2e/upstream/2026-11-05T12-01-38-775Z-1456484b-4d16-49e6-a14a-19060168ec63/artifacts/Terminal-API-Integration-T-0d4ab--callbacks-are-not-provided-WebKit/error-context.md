# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: Terminal.test.ts >> API Integration Tests >> registerLinkProvider >> should work fine when hover and leave callbacks are not provided
- Location: test\playwright\Terminal.test.ts:898:9

# Error details

```
Error: page.evaluate: TypeError: window.term.registerLinkProvider is not a function. (In 'window.term.registerLinkProvider({
          provideLinks: (position, cb) => {
            window.calls.push('provide ' + position);
            if (position === 1) {
              window.calls.push('match 1');
              cb([{
                range: { start: { x: 5, y: 1 }, end: { x: 7, y: 1 } },
                text: 'bar',
                activate: () => window.calls.push('activate')
              }]);
            } else if (position === 2) {
              window.calls.push('match 2');
              cb([{
                range: { start: { x: 5, y: 2 }, end: { x: 7, y: 2 } },
                text: 'bar',
                activate: () => window.calls.push('activate')
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
  806  |         await ctx.page.evaluate(`window.marker = window.term.registerMarker(1)`);
  807  |         await ctx.page.evaluate(`
  808  |         try {
  809  |           window.decoration = window.term.registerDecoration({ marker: window.marker, x: -2 });
  810  |         } catch (e) {
  811  |           window.throwMessage = e.message;
  812  |         }
  813  |       `);
  814  |         await pollFor(ctx.page, 'window.throwMessage', 'This API only accepts positive integers');
  815  |       });
  816  |     });
  817  |     test.describe('overviewRulerDecorations', () => {
  818  |       test('should not add an overview ruler when width is not set', async () => {
  819  |         await openTerminal(ctx, { scrollbar: { overviewRuler: {} } });
  820  |         await ctx.page.evaluate(`window.marker1 = window.term.registerMarker(1)`);
  821  |         await ctx.page.evaluate(`window.marker2 = window.term.registerMarker(2)`);
  822  |         await ctx.page.evaluate(`window.term.registerDecoration({ marker: window.marker1, overviewRulerOptions: { color: 'red', position: 'full' } })`);
  823  |         await ctx.page.evaluate(`window.term.registerDecoration({ marker: window.marker2, overviewRulerOptions: { color: 'blue', position: 'full' } })`);
  824  |         await pollFor(ctx.page, `document.querySelectorAll('.xterm-decoration-overview-ruler').length`, 0);
  825  |       });
  826  |       test('should add an overview ruler when width is set', async () => {
  827  |         await openTerminal(ctx, { scrollbar: { width: 15, overviewRuler: {} } });
  828  |         await ctx.page.evaluate(`window.marker1 = window.term.registerMarker(1)`);
  829  |         await ctx.page.evaluate(`window.marker2 = window.term.registerMarker(2)`);
  830  |         await ctx.page.evaluate(`window.term.registerDecoration({ marker: window.marker1, overviewRulerOptions: { color: 'red', position: 'full' } })`);
  831  |         await ctx.page.evaluate(`window.term.registerDecoration({ marker: window.marker2, overviewRulerOptions: { color: 'blue', position: 'full' } })`);
  832  |         await pollFor(ctx.page, `document.querySelectorAll('.xterm-decoration-overview-ruler').length`, 1);
  833  |       });
  834  |     });
  835  |   });
  836  | 
  837  |   test.describe('registerLinkProvider', () => {
  838  |     test('should fire provideLinks when hovering cells', async () => {
  839  |       await openTerminal(ctx);
  840  |       // Focus the terminal as the cursor will show and trigger a rerender, which can clear the
  841  |       // active link
  842  |       await ctx.proxy.focus();
  843  |       await ctx.page.evaluate(`
  844  |         window.calls = [];
  845  |         window.disposable = window.term.registerLinkProvider({
  846  |           provideLinks: (position, cb) => {
  847  |             calls.push(position);
  848  |             cb(undefined);
  849  |           }
  850  |         });
  851  |       `);
  852  |       const dims = await getDimensions();
  853  |       await moveMouseCell(dims, 1, 1);
  854  |       await moveMouseCell(dims, 2, 2);
  855  |       await moveMouseCell(dims, 10, 4);
  856  |       await pollFor(ctx.page, `window.calls`, [1, 2, 4]);
  857  |       await ctx.page.evaluate(`window.disposable.dispose()`);
  858  |     });
  859  | 
  860  |     test('should fire hover and leave events on the link', async () => {
  861  |       await openTerminal(ctx);
  862  |       // Focus the terminal as the cursor will show and trigger a rerender, which can clear the
  863  |       // active link
  864  |       await ctx.page.evaluate('window.term.focus()');
  865  |       await ctx.proxy.write('foo bar baz');
  866  |       // Wait for renderer to catch up as links are cleared on render
  867  |       await pollFor(ctx.page, `document.querySelector('.xterm-rows').textContent`, 'foo bar baz ');
  868  |       await ctx.page.evaluate(`
  869  |         window.calls = [];
  870  |         window.disposable = window.term.registerLinkProvider({
  871  |           provideLinks: (position, cb) => {
  872  |             window.calls.push('provide ' + position);
  873  |             if (position === 1) {
  874  |               window.calls.push('match');
  875  |               cb([{
  876  |                 range: { start: { x: 5, y: 1 }, end: { x: 7, y: 1 } },
  877  |                 text: 'bar',
  878  |                 activate: () => window.calls.push('activate'),
  879  |                 hover: () => window.calls.push('hover'),
  880  |                 leave: () => window.calls.push('leave')
  881  |               }]);
  882  |             }
  883  |           }
  884  |         });
  885  |       `);
  886  |       const dims = await getDimensions();
  887  |       await moveMouseCell(dims, 5, 1);
  888  |       await timeout(100);
  889  |       await moveMouseCell(dims, 4, 1);
  890  |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'match', 'hover', 'leave' ]);
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
> 906  |       await ctx.page.evaluate(`
       |                      ^ Error: page.evaluate: TypeError: window.term.registerLinkProvider is not a function. (In 'window.term.registerLinkProvider({
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
  991  |       await ctx.page.evaluate(`
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
```