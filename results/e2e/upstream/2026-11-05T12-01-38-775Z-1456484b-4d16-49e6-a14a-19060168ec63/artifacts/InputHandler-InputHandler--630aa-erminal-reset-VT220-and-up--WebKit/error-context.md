# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: InputHandler.test.ts >> InputHandler Integration Tests >> CSI >> CSI ! p - DECSTR: Soft terminal reset, VT220 and up.
- Location: test\playwright\InputHandler.test.ts:1464:9

# Error details

```
Error: page.evaluate: TypeError: undefined is not an object (evaluating 'window.term._core._bufferService.buffer')
```

# Page snapshot

```yaml
- generic [ref=e6]:
  - textbox "Terminal input" [ref=e7]
  - text: WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW
```

# Test source

```ts
  1372 |         const cell = await (await ctx.proxy.buffer.active.getLine(0))!.getCell(0);
  1373 |         deepStrictEqual(await cell!.isFgPalette(), true);
  1374 |         deepStrictEqual(await cell!.getFgColor(), 123);
  1375 |       });
  1376 |       test('Ps = 48:2:Pi:Pr:Pg:Pb - Set background color using RGB values (colon separator)', async () => {
  1377 |         await ctx.proxy.write('\x1b[48:2::18:52:86m#');
  1378 |         const cell = await (await ctx.proxy.buffer.active.getLine(0))!.getCell(0);
  1379 |         deepStrictEqual(await cell!.isBgRGB(), true);
  1380 |         deepStrictEqual(await cell!.getBgColor(), 0x123456);
  1381 |       });
  1382 |       test('Ps = 48:5:Ps - Set background color to Ps using indexed color (colon separator)', async () => {
  1383 |         await ctx.proxy.write('\x1b[48:5:200m#');
  1384 |         const cell = await (await ctx.proxy.buffer.active.getLine(0))!.getCell(0);
  1385 |         deepStrictEqual(await cell!.isBgPalette(), true);
  1386 |         deepStrictEqual(await cell!.getBgColor(), 200);
  1387 |       });
  1388 |       test('Ps = 38;2;Pr;Pg;Pb - Set foreground color using RGB values (semicolon separator)', async () => {
  1389 |         await ctx.proxy.write('\x1b[38;2;171;205;239m#');
  1390 |         const cell = await (await ctx.proxy.buffer.active.getLine(0))!.getCell(0);
  1391 |         deepStrictEqual(await cell!.isFgRGB(), true);
  1392 |         deepStrictEqual(await cell!.getFgColor(), 0xabcdef);
  1393 |       });
  1394 |       test('Ps = 48;2;Pr;Pg;Pb - Set background color using RGB values (semicolon separator)', async () => {
  1395 |         await ctx.proxy.write('\x1b[48;2;18;52;86m#');
  1396 |         const cell = await (await ctx.proxy.buffer.active.getLine(0))!.getCell(0);
  1397 |         deepStrictEqual(await cell!.isBgRGB(), true);
  1398 |         deepStrictEqual(await cell!.getBgColor(), 0x123456);
  1399 |       });
  1400 |     });
  1401 |     test.skip('CSI > Pp [; Pv] m - XTMODKEYS: Set/reset key modifier options, xterm', () => {
  1402 |       // TODO: Implement
  1403 |     });
  1404 |     test.skip('CSI ? Pp m - XTQMODKEYS: Query key modifier options, xterm', () => {
  1405 |       // TODO: Implement
  1406 |     });
  1407 |     test.describe('CSI Ps n - DSR: Device Status Report', () => {
  1408 |       test('Status Report - CSI 5 n', async () => {
  1409 |         await ctx.proxy.write('\x1b[5n');
  1410 |         deepStrictEqual(recordedData, ['\x1b[0n']);
  1411 |       });
  1412 | 
  1413 |       test('Report Cursor Position (CPR) - CSI 6 n', async () => {
  1414 |         await ctx.proxy.write('\n\nfoo');
  1415 |         await pollFor(ctx.page, async () => [
  1416 |           await ctx.proxy.buffer.active.cursorY,
  1417 |           await ctx.proxy.buffer.active.cursorX
  1418 |         ], [2, 3]);
  1419 |         await ctx.proxy.write('\x1b[6n');
  1420 |         deepStrictEqual(recordedData, ['\x1b[3;4R']);
  1421 |       });
  1422 | 
  1423 |       test('Report Cursor Position (DECXCPR) - CSI ? 6 n', async () => {
  1424 |         await ctx.proxy.write('\n\nfoo');
  1425 |         await pollFor(ctx.page, async () => [
  1426 |           await ctx.proxy.buffer.active.cursorY,
  1427 |           await ctx.proxy.buffer.active.cursorX
  1428 |         ], [2, 3]);
  1429 |         await ctx.proxy.write('\x1b[?6n');
  1430 |         deepStrictEqual(recordedData, ['\x1b[?3;4R']);
  1431 |       });
  1432 |     });
  1433 |     test.skip('CSI > Ps n - Disable key modifier options, xterm', () => {
  1434 |       // TODO: Implement
  1435 |     });
  1436 |     test.describe('CSI ? Ps n - DECDSR: Device Status Report (DEC-specific)', () => {
  1437 |       test('Color Scheme Query - CSI ? 996 n (dark theme)', async () => {
  1438 |         // Default theme has dark background (#000000) and light foreground (#ffffff)
  1439 |         await ctx.proxy.write('\x1b[?996n');
  1440 |         deepStrictEqual(recordedData, ['\x1b[?997;1n']);
  1441 |       });
  1442 | 
  1443 |       test('Color Scheme Query - CSI ? 996 n (light theme)', async () => {
  1444 |         recordedData.length = 0;
  1445 |         await ctx.page.evaluate(`window.term.options.theme = { background: '#ffffff', foreground: '#000000' }`);
  1446 |         await ctx.proxy.write('\x1b[?996n');
  1447 |         deepStrictEqual(recordedData, ['\x1b[?997;2n']);
  1448 |         // Restore default theme
  1449 |         await ctx.page.evaluate(`window.term.options.theme = { background: '#000000', foreground: '#ffffff' }`);
  1450 |       });
  1451 | 
  1452 |       test('Color Scheme Query disabled via vtExtensions.colorSchemeQuery', async () => {
  1453 |         recordedData.length = 0;
  1454 |         await ctx.page.evaluate(`window.term.options.vtExtensions = { colorSchemeQuery: false }`);
  1455 |         await ctx.proxy.write('\x1b[?996n');
  1456 |         deepStrictEqual(recordedData, []);
  1457 |         // Re-enable
  1458 |         await ctx.page.evaluate(`window.term.options.vtExtensions = { colorSchemeQuery: true }`);
  1459 |       });
  1460 |     });
  1461 |     test.skip('CSI > Ps p - XTSMPOINTER: Set resource value pointerMode, xterm', () => {
  1462 |       // TODO: Implement
  1463 |     });
  1464 |     test('CSI ! p - DECSTR: Soft terminal reset, VT220 and up.', async () => {
  1465 |       const rows = await ctx.proxy.rows;
  1466 |       await ctx.proxy.write('\x1b[4h\x1b[?6h\x1b[3;5r');
  1467 |       await pollFor(ctx.page, async () => (await ctx.proxy.modes).insertMode, true);
  1468 |       await pollFor(ctx.page, async () => (await ctx.proxy.modes).originMode, true);
  1469 |       await ctx.proxy.write('\x1b[!p');
  1470 |       await pollFor(ctx.page, async () => (await ctx.proxy.modes).insertMode, false);
  1471 |       await pollFor(ctx.page, async () => (await ctx.proxy.modes).originMode, false);
> 1472 |       await pollFor(ctx.page, () => ctx.page.evaluate(`({ top: window.term._core._bufferService.buffer.scrollTop, bottom: window.term._core._bufferService.buffer.scrollBottom })`), { top: 0, bottom: rows - 1 });
       |                                              ^ Error: page.evaluate: TypeError: undefined is not an object (evaluating 'window.term._core._bufferService.buffer')
  1473 |     });
  1474 |     test.skip('CSI Pl ; Pc " p - DECSCL: Set conformance level, VT220 and up.', () => {
  1475 |       // TODO: Implement
  1476 |     });
  1477 |     test('CSI Ps $ p - DECRQM: Request ANSI mode', async () => {
  1478 |       await ctx.proxy.write('\x1b[4h');
  1479 |       recordedData.length = 0;
  1480 |       await ctx.proxy.write('\x1b[4$p');
  1481 |       deepStrictEqual(recordedData, ['\x1b[4;1$y']);
  1482 |       await ctx.proxy.write('\x1b[4l');
  1483 |       recordedData.length = 0;
  1484 |       await ctx.proxy.write('\x1b[4$p');
  1485 |       deepStrictEqual(recordedData, ['\x1b[4;2$y']);
  1486 |       await ctx.proxy.write('\x1b[20h');
  1487 |       recordedData.length = 0;
  1488 |       await ctx.proxy.write('\x1b[20$p');
  1489 |       deepStrictEqual(recordedData, ['\x1b[20;1$y']);
  1490 |       await ctx.proxy.write('\x1b[20l');
  1491 |     });
  1492 |     test('CSI ? Ps $ p - Request DEC private mode (DECRQM).', async () => {
  1493 |       await ctx.proxy.write('\x1b[?1h');
  1494 |       recordedData.length = 0;
  1495 |       await ctx.proxy.write('\x1b[?1$p');
  1496 |       deepStrictEqual(recordedData, ['\x1b[?1;1$y']);
  1497 |       await ctx.proxy.write('\x1b[?1l');
  1498 |       recordedData.length = 0;
  1499 |       await ctx.proxy.write('\x1b[?1$p');
  1500 |       deepStrictEqual(recordedData, ['\x1b[?1;2$y']);
  1501 |     });
  1502 |     test.skip('CSI [Pm] # p - Push video attributes onto stack (XTPUSHSGR), xterm.  This is an alias for CSI # { , used to work around language limitations of C#.', async () => {
  1503 |       // TODO: Implement
  1504 |     });
  1505 |     test('CSI > Ps q - Report xterm name and version (XTVERSION).', async () => {
  1506 |       await ctx.proxy.write('\x1b[>q');
  1507 |       ok(recordedData.length === 1);
  1508 |       ok(recordedData[0].startsWith('\x1bP>|xterm.js('));
  1509 |       ok(recordedData[0].endsWith('\x1b\\'));
  1510 |     });
  1511 |     test.skip('CSI Ps q - Load LEDs (DECLL), VT100.', async () => {
  1512 |       // TODO: Implement
  1513 |     });
  1514 |     test('CSI Ps SP q - Set cursor style (DECSCUSR), VT520.', async () => {
  1515 |       const getCursorMode = async () => ctx.proxy.core.evaluate(([core]) => {
  1516 |         const modes = core.coreService.decPrivateModes;
  1517 |         return { style: modes.cursorStyle, blink: modes.cursorBlink };
  1518 |       });
  1519 |       await ctx.proxy.write('\x1b[1 q');
  1520 |       deepStrictEqual(await getCursorMode(), { style: 'block', blink: true });
  1521 |       await ctx.proxy.write('\x1b[2 q');
  1522 |       deepStrictEqual(await getCursorMode(), { style: 'block', blink: false });
  1523 |       await ctx.proxy.write('\x1b[3 q');
  1524 |       deepStrictEqual(await getCursorMode(), { style: 'underline', blink: true });
  1525 |       await ctx.proxy.write('\x1b[4 q');
  1526 |       deepStrictEqual(await getCursorMode(), { style: 'underline', blink: false });
  1527 |       await ctx.proxy.write('\x1b[5 q');
  1528 |       deepStrictEqual(await getCursorMode(), { style: 'bar', blink: true });
  1529 |       await ctx.proxy.write('\x1b[6 q');
  1530 |       deepStrictEqual(await getCursorMode(), { style: 'bar', blink: false });
  1531 |       await ctx.proxy.write('\x1b[0 q');
  1532 |       deepStrictEqual(await getCursorMode(), { style: undefined, blink: undefined });
  1533 |     });
  1534 |     test('CSI Ps " q - Select character protection attribute (DECSCA), VT220.', async () => {
  1535 |       await ctx.proxy.write('\x1b[1"q');
  1536 |       await ctx.proxy.write('PROT');
  1537 |       await ctx.proxy.write('\x1b[2"q');
  1538 |       await ctx.proxy.write('open');
  1539 |       await pollFor(ctx.page, () => getLinesAsArray(1), ['PROTopen']);
  1540 |       await ctx.proxy.write('\x1b[1;1H\x1b[?2K');
  1541 |       await pollFor(ctx.page, () => getLinesAsArray(1), ['PROT']);
  1542 |     });
  1543 |     test.skip('CSI # q - Pop video attributes from stack (XTPOPSGR), xterm.', async () => {
  1544 |       // TODO: Implement
  1545 |     });
  1546 |     test('CSI Ps ; Ps r - Set Scrolling Region [top;bottom] (default = full size of window) (DECSTBM), VT100.', async () => {
  1547 |       const rows = await ctx.proxy.rows;
  1548 |       await ctx.proxy.write('\x1b[2;4r');
  1549 |       await pollFor(ctx.page, () => ctx.page.evaluate(`({ top: window.term._core._bufferService.buffer.scrollTop, bottom: window.term._core._bufferService.buffer.scrollBottom })`), { top: 1, bottom: 3 });
  1550 |       await ctx.proxy.write('\x1b[r');
  1551 |       await pollFor(ctx.page, () => ctx.page.evaluate(`({ top: window.term._core._bufferService.buffer.scrollTop, bottom: window.term._core._bufferService.buffer.scrollBottom })`), { top: 0, bottom: rows - 1 });
  1552 |     });
  1553 |     test.skip('CSI ? Pm r - Restore DEC Private Mode Values (XTRESTORE), xterm.', async () => {
  1554 |       // TODO: Implement
  1555 |     });
  1556 |     test.skip('CSI Pt ; Pl ; Pb ; Pr ; Pm $ r - Change Attributes in Rectangular Area (DECCARA), VT400 and up.', async () => {
  1557 |       // TODO: Implement
  1558 |     });
  1559 |     test('CSI s - Save cursor, available only when DECLRMM is disabled (SCOSC, also ANSI.SYS).', async () => {
  1560 |       await ctx.proxy.write('\x1b[3;4H');
  1561 |       await ctx.proxy.write('\x1b[s');
  1562 |       await ctx.proxy.write('\x1b[1;1H');
  1563 |       await ctx.proxy.write('\x1b[u');
  1564 |       await pollFor(ctx.page, () => getCursor(), { col: 3, row: 2 });
  1565 |     });
  1566 |     test.skip('CSI Pl ; Pr s - Set left and right margins (DECSLRM), VT420 and up.', async () => {
  1567 |       // TODO: Implement
  1568 |     });
  1569 |     test.skip('CSI > Ps s - Set/reset shift-escape options (XTSHIFTESCAPE), xterm.', async () => {
  1570 |       // TODO: Implement
  1571 |     });
  1572 |     test.skip('CSI ? Pm s - Save DEC Private Mode Values (XTSAVE), xterm.  Ps values are the same as for DECSET.', async () => {
```