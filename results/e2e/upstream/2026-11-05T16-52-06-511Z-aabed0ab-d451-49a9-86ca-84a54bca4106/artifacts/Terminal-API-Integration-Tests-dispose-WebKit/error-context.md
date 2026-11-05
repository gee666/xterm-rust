# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: Terminal.test.ts >> API Integration Tests >> dispose
- Location: test\playwright\Terminal.test.ts:747:7

# Error details

```
Error: page.evaluate: TypeError: undefined is not an object (evaluating 'window.term._core._store._isDisposed')
```

# Test source

```ts
  657 |       strictEqual(await ctx.page.evaluate(`window.term.buffer.alternate.getLine(0)`), undefined);
  658 |     });
  659 |   });
  660 | 
  661 |   test.describe('modes', () => {
  662 |     test.beforeEach(() => openTerminal(ctx));
  663 |     test('defaults', async () => {
  664 |       deepStrictEqual(await ctx.proxy.modes, {
  665 |         applicationCursorKeysMode: false,
  666 |         applicationKeypadMode: false,
  667 |         bracketedPasteMode: false,
  668 |         insertMode: false,
  669 |         mouseTrackingMode: 'none',
  670 |         originMode: false,
  671 |         reverseWraparoundMode: false,
  672 |         sendFocusMode: false,
  673 |         showCursor: true,
  674 |         synchronizedOutputMode: false,
  675 |         win32InputMode: false,
  676 |         wraparoundMode: true
  677 |       });
  678 |     });
  679 |     test('applicationCursorKeysMode', async () => {
  680 |       await ctx.proxy.write('\x1b[?1h');
  681 |       strictEqual((await ctx.proxy.modes).applicationCursorKeysMode, true);
  682 |       await ctx.proxy.write('\x1b[?1l');
  683 |       strictEqual((await ctx.proxy.modes).applicationCursorKeysMode, false);
  684 |     });
  685 |     test('applicationKeypadMode', async () => {
  686 |       await ctx.proxy.write('\x1b[?66h');
  687 |       strictEqual((await ctx.proxy.modes).applicationKeypadMode, true);
  688 |       await ctx.proxy.write('\x1b[?66l');
  689 |       strictEqual((await ctx.proxy.modes).applicationKeypadMode, false);
  690 |     });
  691 |     test('bracketedPasteMode', async () => {
  692 |       await ctx.proxy.write('\x1b[?2004h');
  693 |       strictEqual((await ctx.proxy.modes).bracketedPasteMode, true);
  694 |       await ctx.proxy.write('\x1b[?2004l');
  695 |       strictEqual((await ctx.proxy.modes).bracketedPasteMode, false);
  696 |     });
  697 |     test('insertMode', async () => {
  698 |       await ctx.proxy.write('\x1b[4h');
  699 |       strictEqual((await ctx.proxy.modes).insertMode, true);
  700 |       await ctx.proxy.write('\x1b[4l');
  701 |       strictEqual((await ctx.proxy.modes).insertMode, false);
  702 |     });
  703 |     test('mouseTrackingMode', async () => {
  704 |       await ctx.proxy.write('\x1b[?9h');
  705 |       strictEqual((await ctx.proxy.modes).mouseTrackingMode, 'x10');
  706 |       await ctx.proxy.write('\x1b[?9l');
  707 |       strictEqual((await ctx.proxy.modes).mouseTrackingMode, 'none');
  708 |       await ctx.proxy.write('\x1b[?1000h');
  709 |       strictEqual((await ctx.proxy.modes).mouseTrackingMode, 'vt200');
  710 |       await ctx.proxy.write('\x1b[?1000l');
  711 |       strictEqual((await ctx.proxy.modes).mouseTrackingMode, 'none');
  712 |       await ctx.proxy.write('\x1b[?1002h');
  713 |       strictEqual((await ctx.proxy.modes).mouseTrackingMode, 'drag');
  714 |       await ctx.proxy.write('\x1b[?1002l');
  715 |       strictEqual((await ctx.proxy.modes).mouseTrackingMode, 'none');
  716 |       await ctx.proxy.write('\x1b[?1003h');
  717 |       strictEqual((await ctx.proxy.modes).mouseTrackingMode, 'any');
  718 |       await ctx.proxy.write('\x1b[?1003l');
  719 |       strictEqual((await ctx.proxy.modes).mouseTrackingMode, 'none');
  720 |     });
  721 |     test('originMode', async () => {
  722 |       await ctx.proxy.write('\x1b[?6h');
  723 |       strictEqual((await ctx.proxy.modes).originMode, true);
  724 |       await ctx.proxy.write('\x1b[?6l');
  725 |       strictEqual((await ctx.proxy.modes).originMode, false);
  726 |     });
  727 |     test('reverseWraparoundMode', async () => {
  728 |       await ctx.proxy.write('\x1b[?45h');
  729 |       strictEqual((await ctx.proxy.modes).reverseWraparoundMode, true);
  730 |       await ctx.proxy.write('\x1b[?45l');
  731 |       strictEqual((await ctx.proxy.modes).reverseWraparoundMode, false);
  732 |     });
  733 |     test('sendFocusMode', async () => {
  734 |       await ctx.proxy.write('\x1b[?1004h');
  735 |       strictEqual((await ctx.proxy.modes).sendFocusMode, true);
  736 |       await ctx.proxy.write('\x1b[?1004l');
  737 |       strictEqual((await ctx.proxy.modes).sendFocusMode, false);
  738 |     });
  739 |     test('wraparoundMode', async () => {
  740 |       await ctx.proxy.write('\x1b[?7h');
  741 |       strictEqual((await ctx.proxy.modes).wraparoundMode, true);
  742 |       await ctx.proxy.write('\x1b[?7l');
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
> 757 |     strictEqual(await ctx.page.evaluate(`window.term._core._store._isDisposed`), true);
      |                                ^ Error: page.evaluate: TypeError: undefined is not an object (evaluating 'window.term._core._store._isDisposed')
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
  843 |       await ctx.page.evaluate(`
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
```