# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: Terminal.test.ts >> API Integration Tests >> registerLinkProvider >> should fire provideLinks when hovering cells
- Location: test\playwright\Terminal.test.ts:838:9

# Error details

```
Error: page.evaluate: TypeError: window.term.registerLinkProvider is not a function. (In 'window.term.registerLinkProvider({
          provideLinks: (position, cb) => {
            calls.push(position);
            cb(undefined);
          }
        })', 'window.term.registerLinkProvider' is undefined)
```

# Page snapshot

```yaml
- generic [ref=e6]:
  - textbox "Terminal input" [active]
  - text: WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW
```

# Test source

```ts
  743 |       strictEqual((await ctx.proxy.modes).wraparoundMode, false);
  744 |     });
  745 |   });
  746 | 
  747 |   test('dispose', async () => {
  748 |     await ctx.page.evaluate(`
  749 |       if ('term' in window) {
  750 |         try {
  751 |           window.term.dispose();
  752 |         } catch {}
  753 |       }
  754 |       window.term = new Terminal();
  755 |       window.term.dispose();
  756 |     `);
  757 |     strictEqual(await ctx.page.evaluate(`window.term._core._store._isDisposed`), true);
  758 |   });
  759 | 
  760 |   test('dispose (opened)', async () => {
  761 |     await openTerminal(ctx);
  762 |     await ctx.page.evaluate(`
  763 |       if ('term' in window) {
  764 |         try {
  765 |           window.term.dispose();
  766 |         } catch {}
  767 |       }
  768 |     `);
  769 |     strictEqual(await ctx.page.evaluate(`window.term._core._store._isDisposed`), true);
  770 |   });
  771 | 
  772 |   test('render when visible after hidden', async () => {
  773 |     await openTerminal(ctx);
  774 |     await ctx.page.evaluate(`
  775 |       if ('term' in window) {
  776 |         try {
  777 |           window.term.dispose();
  778 |         } catch {}
  779 |       }
  780 |     `);
  781 |     await ctx.page.evaluate(`document.querySelector('#terminal-container').style.display='none'`);
  782 |     await ctx.page.evaluate(`window.term = new Terminal()`);
  783 |     await ctx.page.evaluate(`window.term.open(document.querySelector('#terminal-container'))`);
  784 |     await ctx.page.evaluate(`document.querySelector('#terminal-container').style.display=''`);
  785 |     await pollFor(ctx.page, `window.term.dimensions.css.cell.width > 0`, true);
  786 |   });
  787 | 
  788 |   test.describe('registerDecoration', () => {
  789 |     test.describe('bufferDecorations', () => {
  790 |       test('should register decorations and render them when terminal open is called', async () => {
  791 |         await openTerminal(ctx);
  792 |         await ctx.page.evaluate(`window.marker1 = window.term.registerMarker(1)`);
  793 |         await ctx.page.evaluate(`window.marker2 = window.term.registerMarker(2)`);
  794 |         await ctx.page.evaluate(`window.term.registerDecoration({ marker: window.marker1 })`);
  795 |         await ctx.page.evaluate(`window.term.registerDecoration({ marker: window.marker2 })`);
  796 |         await pollFor(ctx.page, `document.querySelectorAll('.xterm-screen .xterm-decoration').length`, 2);
  797 |       });
  798 |       test('should return undefined when the marker has already been disposed of', async () => {
  799 |         await openTerminal(ctx);
  800 |         await ctx.page.evaluate(`window.marker = window.term.registerMarker(1)`);
  801 |         await ctx.page.evaluate(`window.marker.dispose()`);
  802 |         await pollFor(ctx.page, `window.decoration = window.term.registerDecoration({ marker: window.marker });`, undefined);
  803 |       });
  804 |       test('should throw when a negative x offset is provided', async () => {
  805 |         await openTerminal(ctx);
  806 |         await ctx.page.evaluate(`window.marker = window.term.registerMarker(1)`);
  807 |         await ctx.page.evaluate(`
  808 |         try {
  809 |           window.decoration = window.term.registerDecoration({ marker: window.marker, x: -2 });
  810 |         } catch (e) {
  811 |           window.throwMessage = e.message;
  812 |         }
  813 |       `);
  814 |         await pollFor(ctx.page, 'window.throwMessage', 'This API only accepts positive integers');
  815 |       });
  816 |     });
  817 |     test.describe('overviewRulerDecorations', () => {
  818 |       test('should not add an overview ruler when width is not set', async () => {
  819 |         await openTerminal(ctx, { scrollbar: { overviewRuler: {} } });
  820 |         await ctx.page.evaluate(`window.marker1 = window.term.registerMarker(1)`);
  821 |         await ctx.page.evaluate(`window.marker2 = window.term.registerMarker(2)`);
  822 |         await ctx.page.evaluate(`window.term.registerDecoration({ marker: window.marker1, overviewRulerOptions: { color: 'red', position: 'full' } })`);
  823 |         await ctx.page.evaluate(`window.term.registerDecoration({ marker: window.marker2, overviewRulerOptions: { color: 'blue', position: 'full' } })`);
  824 |         await pollFor(ctx.page, `document.querySelectorAll('.xterm-decoration-overview-ruler').length`, 0);
  825 |       });
  826 |       test('should add an overview ruler when width is set', async () => {
  827 |         await openTerminal(ctx, { scrollbar: { width: 15, overviewRuler: {} } });
  828 |         await ctx.page.evaluate(`window.marker1 = window.term.registerMarker(1)`);
  829 |         await ctx.page.evaluate(`window.marker2 = window.term.registerMarker(2)`);
  830 |         await ctx.page.evaluate(`window.term.registerDecoration({ marker: window.marker1, overviewRulerOptions: { color: 'red', position: 'full' } })`);
  831 |         await ctx.page.evaluate(`window.term.registerDecoration({ marker: window.marker2, overviewRulerOptions: { color: 'blue', position: 'full' } })`);
  832 |         await pollFor(ctx.page, `document.querySelectorAll('.xterm-decoration-overview-ruler').length`, 1);
  833 |       });
  834 |     });
  835 |   });
  836 | 
  837 |   test.describe('registerLinkProvider', () => {
  838 |     test('should fire provideLinks when hovering cells', async () => {
  839 |       await openTerminal(ctx);
  840 |       // Focus the terminal as the cursor will show and trigger a rerender, which can clear the
  841 |       // active link
  842 |       await ctx.proxy.focus();
> 843 |       await ctx.page.evaluate(`
      |                      ^ Error: page.evaluate: TypeError: window.term.registerLinkProvider is not a function. (In 'window.term.registerLinkProvider({
  844 |         window.calls = [];
  845 |         window.disposable = window.term.registerLinkProvider({
  846 |           provideLinks: (position, cb) => {
  847 |             calls.push(position);
  848 |             cb(undefined);
  849 |           }
  850 |         });
  851 |       `);
  852 |       const dims = await getDimensions();
  853 |       await moveMouseCell(dims, 1, 1);
  854 |       await moveMouseCell(dims, 2, 2);
  855 |       await moveMouseCell(dims, 10, 4);
  856 |       await pollFor(ctx.page, `window.calls`, [1, 2, 4]);
  857 |       await ctx.page.evaluate(`window.disposable.dispose()`);
  858 |     });
  859 | 
  860 |     test('should fire hover and leave events on the link', async () => {
  861 |       await openTerminal(ctx);
  862 |       // Focus the terminal as the cursor will show and trigger a rerender, which can clear the
  863 |       // active link
  864 |       await ctx.page.evaluate('window.term.focus()');
  865 |       await ctx.proxy.write('foo bar baz');
  866 |       // Wait for renderer to catch up as links are cleared on render
  867 |       await pollFor(ctx.page, `document.querySelector('.xterm-rows').textContent`, 'foo bar baz ');
  868 |       await ctx.page.evaluate(`
  869 |         window.calls = [];
  870 |         window.disposable = window.term.registerLinkProvider({
  871 |           provideLinks: (position, cb) => {
  872 |             window.calls.push('provide ' + position);
  873 |             if (position === 1) {
  874 |               window.calls.push('match');
  875 |               cb([{
  876 |                 range: { start: { x: 5, y: 1 }, end: { x: 7, y: 1 } },
  877 |                 text: 'bar',
  878 |                 activate: () => window.calls.push('activate'),
  879 |                 hover: () => window.calls.push('hover'),
  880 |                 leave: () => window.calls.push('leave')
  881 |               }]);
  882 |             }
  883 |           }
  884 |         });
  885 |       `);
  886 |       const dims = await getDimensions();
  887 |       await moveMouseCell(dims, 5, 1);
  888 |       await timeout(100);
  889 |       await moveMouseCell(dims, 4, 1);
  890 |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'match', 'hover', 'leave' ]);
  891 |       await moveMouseCell(dims, 7, 1);
  892 |       await timeout(100);
  893 |       await moveMouseCell(dims, 8, 1);
  894 |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'match', 'hover', 'leave', 'hover', 'leave']);
  895 |       await ctx.page.evaluate(`window.disposable.dispose()`);
  896 |     });
  897 | 
  898 |     test('should work fine when hover and leave callbacks are not provided', async () => {
  899 |       await openTerminal(ctx);
  900 |       // Focus the terminal as the cursor will show and trigger a rerender, which can clear the
  901 |       // active link
  902 |       await ctx.page.evaluate('window.term.focus()');
  903 |       await ctx.proxy.write('foo bar baz');
  904 |       // Wait for renderer to catch up as links are cleared on render
  905 |       await pollFor(ctx.page, `document.querySelector('.xterm-rows').textContent`, 'foo bar baz ');
  906 |       await ctx.page.evaluate(`
  907 |         window.calls = [];
  908 |         window.disposable = window.term.registerLinkProvider({
  909 |           provideLinks: (position, cb) => {
  910 |             window.calls.push('provide ' + position);
  911 |             if (position === 1) {
  912 |               window.calls.push('match 1');
  913 |               cb([{
  914 |                 range: { start: { x: 5, y: 1 }, end: { x: 7, y: 1 } },
  915 |                 text: 'bar',
  916 |                 activate: () => window.calls.push('activate')
  917 |               }]);
  918 |             } else if (position === 2) {
  919 |               window.calls.push('match 2');
  920 |               cb([{
  921 |                 range: { start: { x: 5, y: 2 }, end: { x: 7, y: 2 } },
  922 |                 text: 'bar',
  923 |                 activate: () => window.calls.push('activate')
  924 |               }]);
  925 |             }
  926 |           }
  927 |         });
  928 |       `);
  929 |       const dims = await getDimensions();
  930 |       await moveMouseCell(dims, 5, 1);
  931 |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'match 1']);
  932 |       await moveMouseCell(dims, 4, 2);
  933 |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'match 1', 'provide 2', 'match 2']);
  934 |       await moveMouseCell(dims, 7, 1);
  935 |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'match 1', 'provide 2', 'match 2', 'provide 1', 'match 1']);
  936 |       await moveMouseCell(dims, 6, 2);
  937 |       await pollFor(ctx.page, `window.calls`, ['provide 1', 'match 1', 'provide 2', 'match 2', 'provide 1', 'match 1', 'provide 2', 'match 2']);
  938 |       await ctx.page.evaluate(`window.disposable.dispose()`);
  939 |     });
  940 | 
  941 |     test('should fire activate events when clicking the link', async () => {
  942 |       await openTerminal(ctx);
  943 |       // Focus the terminal as the cursor will show and trigger a rerender, which can clear the
```