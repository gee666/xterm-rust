# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: InputHandler.test.ts >> InputHandler Integration Tests >> CSI >> CSI Ps ; Ps r - Set Scrolling Region [top;bottom] (default = full size of window) (DECSTBM), VT100.
- Location: test\playwright\InputHandler.test.ts:1546:9

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
  1472 |       await pollFor(ctx.page, () => ctx.page.evaluate(`({ top: window.term._core._bufferService.buffer.scrollTop, bottom: window.term._core._bufferService.buffer.scrollBottom })`), { top: 0, bottom: rows - 1 });
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
> 1549 |       await pollFor(ctx.page, () => ctx.page.evaluate(`({ top: window.term._core._bufferService.buffer.scrollTop, bottom: window.term._core._bufferService.buffer.scrollBottom })`), { top: 1, bottom: 3 });
       |                                              ^ Error: page.evaluate: TypeError: undefined is not an object (evaluating 'window.term._core._bufferService.buffer')
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
  1573 |       // TODO: Implement
  1574 |     });
  1575 |     test.skip('CSI > Pm t - This xterm control sets one or more features of the title modes (XTSMTITLE), xterm.', async () => {
  1576 |       // TODO: Implement
  1577 |     });
  1578 |     test.skip('CSI Ps SP t - Set warning-bell volume (DECSWBV), VT520.', async () => {
  1579 |       // TODO: Implement
  1580 |     });
  1581 |     test.skip('CSI Pt ; Pl ; Pb ; Pr ; Pm $ t - Reverse Attributes in Rectangular Area (DECRARA), VT400 and up.', async () => {
  1582 |       // TODO: Implement
  1583 |     });
  1584 |     test('CSI u - Restore cursor (SCORC, also ANSI.SYS).', async () => {
  1585 |       await ctx.proxy.write('\x1b[4;6H');
  1586 |       await ctx.proxy.write('\x1b[s');
  1587 |       await ctx.proxy.write('\x1b[1;1H');
  1588 |       await ctx.proxy.write('\x1b[u');
  1589 |       await pollFor(ctx.page, () => getCursor(), { col: 5, row: 3 });
  1590 |     });
  1591 |     test.skip('CSI Ps SP u - Set margin-bell volume (DECSMBV), VT520.', async () => {
  1592 |       // TODO: Implement
  1593 |     });
  1594 |     test.skip('CSI Pt ; Pl ; Pb ; Pr ; Pp ; Pt ; Pl ; Pp $ v - Copy Rectangular Area (DECCRA), VT400 and up.', async () => {
  1595 |       // TODO: Implement
  1596 |     });
  1597 |     test.skip('CSI Ps $ w - Request presentation state report (DECRQPSR), VT320 and up.', async () => {
  1598 |       // TODO: Implement
  1599 |     });
  1600 |     test.skip('CSI Pt ; Pl ; Pb ; Pr \' w - Enable Filter Rectangle (DECEFR), VT420 and up.', async () => {
  1601 |       // TODO: Implement
  1602 |     });
  1603 |     test.skip('CSI Ps x - Request Terminal Parameters (DECREQTPARM).', async () => {
  1604 |       // TODO: Implement
  1605 |     });
  1606 |     test.skip('CSI Ps * x - Select Attribute Change Extent (DECSACE), VT420 and up.', async () => {
  1607 |       // TODO: Implement
  1608 |     });
  1609 |     test.skip('CSI Pc ; Pt ; Pl ; Pb ; Pr $ x - Fill Rectangular Area (DECFRA), VT420 and up.', async () => {
  1610 |       // TODO: Implement
  1611 |     });
  1612 |     test.skip('CSI Ps # y - Select checksum extension (XTCHECKSUM), xterm.', async () => {
  1613 |       // TODO: Implement
  1614 |     });
  1615 |     test.skip('CSI Pi ; Pg ; Pt ; Pl ; Pb ; Pr * y - Request Checksum of Rectangular Area (DECRQCRA), VT420 and up.', async () => {
  1616 |       // TODO: Implement
  1617 |     });
  1618 |     test.skip('CSI Ps ; Pu \' z - Enable Locator Reporting (DECELR).', async () => {
  1619 |       // TODO: Implement
  1620 |     });
  1621 |     test.skip('CSI Pt ; Pl ; Pb ; Pr $ z - Erase Rectangular Area (DECERA), VT400 and up.', async () => {
  1622 |       // TODO: Implement
  1623 |     });
  1624 |     test.skip('CSI Pm \' { - Select Locator Events (DECSLE).', async () => {
  1625 |       // TODO: Implement
  1626 |     });
  1627 |     test.skip('CSI [Pm] # { Push video attributes onto stack (XTPUSHSGR), xterm.', async () => {
  1628 |       // TODO: Implement
  1629 |     });
  1630 |     test.skip('CSI Pt ; Pl ; Pb ; Pr $ { - Selective Erase Rectangular Area (DECSERA), VT400 and up.', async () => {
  1631 |       // TODO: Implement
  1632 |     });
  1633 |     test.skip('CSI Pt ; Pl ; Pb ; Pr # | - Report selected graphic rendition (XTREPORTSGR), xterm.', async () => {
  1634 |       // TODO: Implement
  1635 |     });
  1636 |     test.skip('CSI Ps $ | - Select columns per page (DECSCPP), VT340.', async () => {
  1637 |       // TODO: Implement
  1638 |     });
  1639 |     test.skip('CSI Ps \' | - Request Locator Position (DECRQLP).', async () => {
  1640 |       // TODO: Implement
  1641 |     });
  1642 |     test.skip('CSI Ps * | - Select number of lines per screen (DECSNLS), VT420 and up.', async () => {
  1643 |       // TODO: Implement
  1644 |     });
  1645 |     test.skip('CSI # } - Pop video attributes from stack (XTPOPSGR), xterm.', async () => {
  1646 |       // TODO: Implement
  1647 |     });
  1648 |     test('CSI Ps \' } - Insert Ps Column(s) (default = 1) (DECIC), VT420 and up.', async () => {
  1649 |       await ctx.proxy.resize(5, 5);
```