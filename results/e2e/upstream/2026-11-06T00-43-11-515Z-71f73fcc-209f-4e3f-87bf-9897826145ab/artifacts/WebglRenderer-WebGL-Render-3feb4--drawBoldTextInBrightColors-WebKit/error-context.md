# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: WebglRenderer.test.ts >> WebGL Renderer Integration Tests >> foreground 0-7 drawBoldTextInBrightColors
- Location: test\playwright\SharedRendererTests.ts:54:7

# Error details

```
Test timeout of 10000ms exceeded.
```

```
Error: locator.screenshot: Target page, context or browser has been closed
Call log:
  - waiting for locator('#terminal-container .xterm-screen')
    - locator resolved to visible <div class="xterm-screen">…</div>

```

# Page snapshot

```yaml
- generic [ref=e5]:
  - generic [ref=e6]:
    - textbox "Terminal input" [ref=e7]
    - text: WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW
  - generic:
    - generic:
      - generic: ■
      - generic: ■
      - generic: ■
      - generic: ■
      - generic: ■
      - generic: ■
      - generic: ■
      - generic: ■
```

# Test source

```ts
  1427 |         return {
  1428 |           fg: style.color,
  1429 |           bg: style.backgroundColor,
  1430 |           char: chars[col - currentCol] ?? ''
  1431 |         };
  1432 |       }
  1433 |       currentCol = nextCol;
  1434 |     }
  1435 |     return undefined;
  1436 |   }, { col, row });
  1437 | 
  1438 |   if (!info) {
  1439 |     throw new Error(`Cell not found: ${col},${row}`);
  1440 |   }
  1441 |   return info;
  1442 | }
  1443 | 
  1444 | async function getCellDomBackgroundColor(ctx: ITestContext, col: number, row: number): Promise<[number, number, number, number]> {
  1445 |   const info = await getCellDomInfo(ctx, col, row);
  1446 |   return parseCssColor(info.bg);
  1447 | }
  1448 | 
  1449 | async function getCellDomForegroundColor(ctx: ITestContext, col: number, row: number): Promise<[number, number, number, number]> {
  1450 |   const info = await getCellDomInfo(ctx, col, row);
  1451 |   return parseCssColor(info.fg);
  1452 | }
  1453 | 
  1454 | async function getCellDomChar(ctx: ITestContext, col: number, row: number): Promise<string> {
  1455 |   const info = await getCellDomInfo(ctx, col, row);
  1456 |   return info.char;
  1457 | }
  1458 | 
  1459 | enum CellColorPosition {
  1460 |   CENTER = 0,
  1461 |   FIRST = 1
  1462 | }
  1463 | 
  1464 | /**
  1465 |  * Injects shared renderer tests where it's required to re-initialize the terminal for each test.
  1466 |  * This is much slower than just calling `Terminal.reset` but testing some features needs this
  1467 |  * treatment.
  1468 |  */
  1469 | export function injectSharedRendererTestsStandalone(ctx: ISharedRendererTestContext, setupCb: () => Promise<void> | void): void {
  1470 |   const setupTests = ({ shadowDom }: { shadowDom: boolean }): void => {
  1471 |     test.beforeEach(async () => {
  1472 |       // Recreate terminal
  1473 |       await openTerminal(ctx.value, {}, { useShadowDom: shadowDom });
  1474 |       await ctx.value.page.evaluate(`
  1475 |         window.term.options.minimumContrastRatio = 1;
  1476 |         window.term.options.allowTransparency = false;
  1477 |         window.term.options.theme = undefined;
  1478 |       `);
  1479 |       await setupCb();
  1480 |       // Clear the cached screenshot before each test
  1481 |       frameDetails = undefined;
  1482 |     });
  1483 |     test.describe('regression tests', () => {
  1484 |       test('#4790: cursor should not be displayed before focusing', async () => {
  1485 |         const theme: ITheme = {
  1486 |           cursor: '#0000FF'
  1487 |         };
  1488 |         await ctx.value.page.evaluate(`window.term.options.theme = ${JSON.stringify(theme)};`);
  1489 |         await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [0, 0, 0, 255]);
  1490 |         await ctx.value.proxy.focus();
  1491 |         frameDetails = undefined;
  1492 |         await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [0, 0, 255, 255]);
  1493 |         await ctx.value.proxy.blur();
  1494 |         frameDetails = undefined;
  1495 |         await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [0, 0, 0, 255]);
  1496 |       });
  1497 |     });
  1498 |   };
  1499 | 
  1500 |   test.describe('standalone tests', () => {
  1501 |     setupTests({ shadowDom: false });
  1502 |   });
  1503 |   test.describe('standalone tests (Shadow dom)', () => {
  1504 |     setupTests({ shadowDom: true });
  1505 |   });
  1506 | }
  1507 | 
  1508 | /**
  1509 |  * Gets the color of the pixel in the center of a cell.
  1510 |  * @param ctx The test context.
  1511 |  * @param col The 1-based column index to get the color for.
  1512 |  * @param row The 1-based row index to get the color for.
  1513 |  */
  1514 | async function getCellColor(ctx: ITestContext, col: number, row: number, position: CellColorPosition = CellColorPosition.CENTER): Promise<[red: number, green: number, blue: number, alpha: number]> {
  1515 |   frameDetails ??= await getFrameDetails(ctx);
  1516 |   switch (position) {
  1517 |     case CellColorPosition.CENTER:
  1518 |       return getCellColorInner(frameDetails, col, row);
  1519 |     case CellColorPosition.FIRST:
  1520 |       return getCellColorFirstPoint(frameDetails, col, row);
  1521 |   }
  1522 | }
  1523 | 
  1524 | let frameDetails: { cols: number, rows: number, decoded: IImage32 } | undefined = undefined;
  1525 | async function getFrameDetails(ctx: ITestContext): Promise<{ cols: number, rows: number, decoded: IImage32 }> {
  1526 |   const screenshotOptions: LocatorScreenshotOptions | undefined = process.env.DEBUG ? { path: 'out-esbuild-test/playwright/screenshot.png' } : undefined;
> 1527 |   const buffer = await ctx.page.locator('#terminal-container .xterm-screen').screenshot(screenshotOptions);
       |                                                                              ^ Error: locator.screenshot: Target page, context or browser has been closed
  1528 |   frameDetails = {
  1529 |     cols: await ctx.proxy.cols,
  1530 |     rows: await ctx.proxy.rows,
  1531 |     decoded: (await decodePng(new Uint8Array(buffer), { force32: true })).image
  1532 |   };
  1533 |   return frameDetails;
  1534 | }
  1535 | 
  1536 | function getCellColorInner(frameDetails: { cols: number, rows: number, decoded: IImage32 }, col: number, row: number): [red: number, green: number, blue: number, alpha: number] {
  1537 |   const cellSize = {
  1538 |     width: frameDetails.decoded.width / frameDetails.cols,
  1539 |     height: frameDetails.decoded.height / frameDetails.rows
  1540 |   };
  1541 |   const x = Math.floor((col - 1/* 1- to 0-based index */ + 0.5/* middle of cell */) * cellSize.width);
  1542 |   const y = Math.floor((row - 1/* 1- to 0-based index */ + 0.5/* middle of cell */) * cellSize.height);
  1543 |   const i = (y * frameDetails.decoded.width + x) * 4/* 4 channels per pixel */;
  1544 |   return Array.from(frameDetails.decoded.data.slice(i, i + 4)) as [number, number, number, number];
  1545 | }
  1546 | 
  1547 | function getCellColorFirstPoint(frameDetails: { cols: number, rows: number, decoded: IImage32 }, col: number, row: number): [red: number, green: number, blue: number, alpha: number] {
  1548 |   const cellSize = {
  1549 |     width: frameDetails.decoded.width / frameDetails.cols,
  1550 |     height: frameDetails.decoded.height / frameDetails.rows
  1551 |   };
  1552 |   const x = Math.floor((col - 1/* 1- to 0-based index */) * cellSize.width);
  1553 |   const y = Math.floor((row - 1/* 1- to 0-based index */) * cellSize.height);
  1554 |   const i = (y * frameDetails.decoded.width + x) * 4/* 4 channels per pixel */;
  1555 |   return Array.from(frameDetails.decoded.data.slice(i, i + 4)) as [number, number, number, number];
  1556 | }
  1557 | 
  1558 | const COLORS_16_TO_255 = [
  1559 |   '#000000', '#00005f', '#000087', '#0000af', '#0000d7', '#0000ff', '#005f00', '#005f5f', '#005f87', '#005faf', '#005fd7', '#005fff', '#008700', '#00875f', '#008787', '#0087af',
  1560 |   '#0087d7', '#0087ff', '#00af00', '#00af5f', '#00af87', '#00afaf', '#00afd7', '#00afff', '#00d700', '#00d75f', '#00d787', '#00d7af', '#00d7d7', '#00d7ff', '#00ff00', '#00ff5f',
  1561 |   '#00ff87', '#00ffaf', '#00ffd7', '#00ffff', '#5f0000', '#5f005f', '#5f0087', '#5f00af', '#5f00d7', '#5f00ff', '#5f5f00', '#5f5f5f', '#5f5f87', '#5f5faf', '#5f5fd7', '#5f5fff',
  1562 |   '#5f8700', '#5f875f', '#5f8787', '#5f87af', '#5f87d7', '#5f87ff', '#5faf00', '#5faf5f', '#5faf87', '#5fafaf', '#5fafd7', '#5fafff', '#5fd700', '#5fd75f', '#5fd787', '#5fd7af',
  1563 |   '#5fd7d7', '#5fd7ff', '#5fff00', '#5fff5f', '#5fff87', '#5fffaf', '#5fffd7', '#5fffff', '#870000', '#87005f', '#870087', '#8700af', '#8700d7', '#8700ff', '#875f00', '#875f5f',
  1564 |   '#875f87', '#875faf', '#875fd7', '#875fff', '#878700', '#87875f', '#878787', '#8787af', '#8787d7', '#8787ff', '#87af00', '#87af5f', '#87af87', '#87afaf', '#87afd7', '#87afff',
  1565 |   '#87d700', '#87d75f', '#87d787', '#87d7af', '#87d7d7', '#87d7ff', '#87ff00', '#87ff5f', '#87ff87', '#87ffaf', '#87ffd7', '#87ffff', '#af0000', '#af005f', '#af0087', '#af00af',
  1566 |   '#af00d7', '#af00ff', '#af5f00', '#af5f5f', '#af5f87', '#af5faf', '#af5fd7', '#af5fff', '#af8700', '#af875f', '#af8787', '#af87af', '#af87d7', '#af87ff', '#afaf00', '#afaf5f',
  1567 |   '#afaf87', '#afafaf', '#afafd7', '#afafff', '#afd700', '#afd75f', '#afd787', '#afd7af', '#afd7d7', '#afd7ff', '#afff00', '#afff5f', '#afff87', '#afffaf', '#afffd7', '#afffff',
  1568 |   '#d70000', '#d7005f', '#d70087', '#d700af', '#d700d7', '#d700ff', '#d75f00', '#d75f5f', '#d75f87', '#d75faf', '#d75fd7', '#d75fff', '#d78700', '#d7875f', '#d78787', '#d787af',
  1569 |   '#d787d7', '#d787ff', '#d7af00', '#d7af5f', '#d7af87', '#d7afaf', '#d7afd7', '#d7afff', '#d7d700', '#d7d75f', '#d7d787', '#d7d7af', '#d7d7d7', '#d7d7ff', '#d7ff00', '#d7ff5f',
  1570 |   '#d7ff87', '#d7ffaf', '#d7ffd7', '#d7ffff', '#ff0000', '#ff005f', '#ff0087', '#ff00af', '#ff00d7', '#ff00ff', '#ff5f00', '#ff5f5f', '#ff5f87', '#ff5faf', '#ff5fd7', '#ff5fff',
  1571 |   '#ff8700', '#ff875f', '#ff8787', '#ff87af', '#ff87d7', '#ff87ff', '#ffaf00', '#ffaf5f', '#ffaf87', '#ffafaf', '#ffafd7', '#ffafff', '#ffd700', '#ffd75f', '#ffd787', '#ffd7af',
  1572 |   '#ffd7d7', '#ffd7ff', '#ffff00', '#ffff5f', '#ffff87', '#ffffaf', '#ffffd7', '#ffffff', '#080808', '#121212', '#1c1c1c', '#262626', '#303030', '#3a3a3a', '#444444', '#4e4e4e',
  1573 |   '#585858', '#626262', '#6c6c6c', '#767676', '#808080', '#8a8a8a', '#949494', '#9e9e9e', '#a8a8a8', '#b2b2b2', '#bcbcbc', '#c6c6c6', '#d0d0d0', '#dadada', '#e4e4e4', '#eeeeee'
  1574 | ];
  1575 | 
```