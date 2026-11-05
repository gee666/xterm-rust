# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: KittyGraphics.test.ts >> Kitty Graphics Protocol >> Larger image (200x100 multicolor PNG) >> Pixel verification >> renders correct colors at rectangle boundaries
- Location: addons\addon-image\test\KittyGraphics.test.ts:1678:11

# Error details

```
Error: page.evaluate: ReferenceError: Can't find variable: ImageAddon
```

# Page snapshot

```yaml
- generic [ref=e6]:
  - textbox "Terminal input"
  - text: WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW
```

# Test source

```ts
  12  |  * Plugin ctor options.
  13  |  */
  14  | export interface IImageAddonOptions {
  15  |   enableSizeReports: boolean;
  16  |   pixelLimit: number;
  17  |   storageLimit: number;
  18  |   showPlaceholder: boolean;
  19  |   sixelSupport: boolean;
  20  |   sixelScrolling: boolean;
  21  |   sixelPaletteLimit: number;
  22  |   sixelSizeLimit: number;
  23  |   iipSupport: boolean;
  24  |   iipSizeLimit: number;
  25  |   kittySupport: boolean;
  26  |   kittySizeLimit: number;
  27  | }
  28  | 
  29  | // eslint-disable-next-line
  30  | declare const ImageAddon: {
  31  |   new(options?: Partial<IImageAddonOptions>): any;
  32  | };
  33  | 
  34  | interface IDimensions {
  35  |   cellWidth: number;
  36  |   cellHeight: number;
  37  |   width: number;
  38  |   height: number;
  39  | }
  40  | 
  41  | // Kitty graphics test images
  42  | const KITTY_BLACK_1X1_BASE64 = readFileSync('./addons/addon-image/fixture/kitty/black-1x1.png').toString('base64');
  43  | const KITTY_BLACK_1X1_BYTES = Array.from(readFileSync('./addons/addon-image/fixture/kitty/black-1x1.png'));
  44  | const KITTY_RGB_3X1_BASE64 = readFileSync('./addons/addon-image/fixture/kitty/rgb-3x1.png').toString('base64');
  45  | const KITTY_MULTICOLOR_200X100_BASE64 = readFileSync('./addons/addon-image/fixture/kitty/multicolor-200x100.png').toString('base64');
  46  | const KITTY_MULTICOLOR_200X100_BYTES = Array.from(readFileSync('./addons/addon-image/fixture/kitty/multicolor-200x100.png'));
  47  | 
  48  | // Raw RGB pixel data (f=24): 3 bytes per pixel, no header — requires s= and v=
  49  | const RAW_RGB_1X1_BLACK = Buffer.from([0, 0, 0]).toString('base64');
  50  | const RAW_RGB_1X1_RED = Buffer.from([255, 0, 0]).toString('base64');
  51  | const RAW_RGB_3X1 = Buffer.from([
  52  |   255, 0, 0,
  53  |   0, 255, 0,
  54  |   0, 0, 255
  55  | ]).toString('base64');
  56  | const RAW_RGB_2X2 = Buffer.from([
  57  |   255, 0, 0,    0, 255, 0,
  58  |   0, 0, 255,    255, 255, 0
  59  | ]).toString('base64');
  60  | // 5 pixels (1 uint32 block + 1 remainder) — tests block+tail boundary
  61  | const RAW_RGB_5X1 = Buffer.from([
  62  |   255, 0, 0,
  63  |   0, 255, 0,
  64  |   0, 0, 255,
  65  |   255, 255, 0,
  66  |   255, 0, 255
  67  | ]).toString('base64');
  68  | // 8 pixels (2 full uint32 blocks, 0 remainder) — tests multi-block path
  69  | const RAW_RGB_4X2 = Buffer.from([
  70  |   255, 0, 0,    0, 255, 0,    0, 0, 255,    255, 255, 0,
  71  |   255, 0, 255,  0, 255, 255,  128, 128, 128, 255, 255, 255
  72  | ]).toString('base64');
  73  | 
  74  | // Raw RGBA pixel data (f=32): 4 bytes per pixel, no header — requires s= and v=
  75  | const RAW_RGBA_1X1_WHITE = Buffer.from([255, 255, 255, 255]).toString('base64');
  76  | const RAW_RGBA_1X1_RED = Buffer.from([255, 0, 0, 255]).toString('base64');
  77  | const RAW_RGBA_1X1_TRANSPARENT = Buffer.from([0, 0, 0, 0]).toString('base64');
  78  | const RAW_RGBA_3X1 = Buffer.from([
  79  |   255, 0, 0, 255,
  80  |   0, 255, 0, 255,
  81  |   0, 0, 255, 255
  82  | ]).toString('base64');
  83  | const RAW_RGBA_2X2 = Buffer.from([
  84  |   255, 0, 0, 255,    0, 255, 0, 255,
  85  |   0, 0, 255, 255,    255, 255, 0, 255
  86  | ]).toString('base64');
  87  | // 5 pixels — tests RGBA zero-copy with non-power-of-2 count
  88  | const RAW_RGBA_5X1 = Buffer.from([
  89  |   255, 0, 0, 255,
  90  |   0, 255, 0, 255,
  91  |   0, 0, 255, 255,
  92  |   255, 255, 0, 255,
  93  |   255, 0, 255, 255
  94  | ]).toString('base64');
  95  | 
  96  | let ctx: ITestContext;
  97  | test.beforeAll(async ({ browser }) => {
  98  |   ctx = await createTestContext(browser);
  99  |   await openTerminal(ctx, { cols: 80, rows: 24 });
  100 | });
  101 | test.afterAll(async () => await ctx.page.close());
  102 | 
  103 | test.describe('Kitty Graphics Protocol', () => {
  104 |   // TODO: Add tests for larger images with various dimensions
  105 |   // TODO: Add tests for virtual placement (U=1)
  106 |   // TODO: Add tests for animation frames
  107 |   // TODO: Add performance tests for streaming large images
  108 |   // TODO: Implement cursor movement per Kitty spec - cursor should move by cols/rows after placement (unless C=1)
  109 |   // TODO: Distinguish lowercase delete selectors (placement only) from uppercase (placement + free data)
  110 | 
  111 |   test.beforeEach(async ({}, testInfo) => {
> 112 |     await ctx.page.evaluate(`
      |                    ^ Error: page.evaluate: ReferenceError: Can't find variable: ImageAddon
  113 |       window.term.reset()
  114 |       window.imageAddon?.dispose();
  115 |       window.imageAddon = new ImageAddon({ sixelPaletteLimit: 512 });
  116 |       window.term.loadAddon(window.imageAddon);
  117 |     `);
  118 |   });
  119 | 
  120 |   test.describe('Basic transmission and storage', () => {
  121 |     test('stores 1x1 black PNG with a=T (transmit and display)', async () => {
  122 |       const seq = `\x1b_Ga=T,f=100;${KITTY_BLACK_1X1_BASE64}\x1b\\`;
  123 |       await ctx.proxy.write(seq);
  124 |       await timeout(100);
  125 |       strictEqual(await getImageStorageLength(), 1);
  126 |       deepStrictEqual(await getOrigSize(1), [1, 1]);
  127 |     });
  128 | 
  129 |     test('stores 3x1 RGB PNG with a=T', async () => {
  130 |       const seq = `\x1b_Ga=T,f=100;${KITTY_RGB_3X1_BASE64}\x1b\\`;
  131 |       await ctx.proxy.write(seq);
  132 |       await timeout(100);
  133 |       strictEqual(await getImageStorageLength(), 1);
  134 |       deepStrictEqual(await getOrigSize(1), [3, 1]);
  135 |     });
  136 | 
  137 |     test('transmit only (a=t) does not display but stores in handler', async () => {
  138 |       const seq = `\x1b_Ga=t,f=100;${KITTY_BLACK_1X1_BASE64}\x1b\\`;
  139 |       await ctx.proxy.write(seq);
  140 |       await timeout(100);
  141 |       strictEqual(await ctx.page.evaluate(`window.imageAddon._handlers.get('kitty').images.size`), 1);
  142 |     });
  143 | 
  144 |     test('uses specified image ID', async () => {
  145 |       const seq = `\x1b_Ga=t,f=100,i=42;${KITTY_BLACK_1X1_BASE64}\x1b\\`;
  146 |       await ctx.proxy.write(seq);
  147 |       await timeout(100);
  148 |       strictEqual(await ctx.page.evaluate(`window.imageAddon._handlers.get('kitty').images.has(42)`), true);
  149 |       strictEqual(await ctx.page.evaluate(`window.imageAddon._handlers.get('kitty').images.has(1)`), false);
  150 |     });
  151 | 
  152 |     test('assigns auto-incrementing IDs when not specified', async () => {
  153 |       await ctx.proxy.write(`\x1b_Ga=t,f=100;${KITTY_BLACK_1X1_BASE64}\x1b\\`);
  154 |       await ctx.proxy.write(`\x1b_Ga=t,f=100;${KITTY_RGB_3X1_BASE64}\x1b\\`);
  155 |       await timeout(100);
  156 |       strictEqual(await ctx.page.evaluate(`window.imageAddon._handlers.get('kitty').images.size`), 2);
  157 |       strictEqual(await ctx.page.evaluate(`window.imageAddon._handlers.get('kitty').images.has(1)`), true);
  158 |       strictEqual(await ctx.page.evaluate(`window.imageAddon._handlers.get('kitty').images.has(2)`), true);
  159 |     });
  160 | 
  161 |     test('defaults to transmit action when action is omitted', async () => {
  162 |       const seq = `\x1b_Gf=100;${KITTY_BLACK_1X1_BASE64}\x1b\\`;
  163 |       await ctx.proxy.write(seq);
  164 |       await timeout(100);
  165 |       strictEqual(await ctx.page.evaluate(`window.imageAddon._handlers.get('kitty').images.size`), 1);
  166 |     });
  167 | 
  168 |     test('ignores command when action is empty string', async () => {
  169 |       const seq = `\x1b_Ga=,f=100;${KITTY_BLACK_1X1_BASE64}\x1b\\`;
  170 |       await ctx.proxy.write(seq);
  171 |       await timeout(100);
  172 |       strictEqual(await ctx.page.evaluate(`window.imageAddon._handlers.get('kitty').images.size`), 0);
  173 |     });
  174 |   });
  175 | 
  176 |   test.describe('Chunked transmission', () => {
  177 |     test('handles chunked transmission (m=1)', async () => {
  178 |       const half = Math.floor(KITTY_BLACK_1X1_BASE64.length / 2);
  179 |       const part1 = KITTY_BLACK_1X1_BASE64.substring(0, half);
  180 |       const part2 = KITTY_BLACK_1X1_BASE64.substring(half);
  181 | 
  182 |       const seq1 = `\x1b_Ga=T,f=100,i=99,m=1;${part1}\x1b\\`;
  183 |       const seq2 = `\x1b_Ga=T,f=100,i=99;${part2}\x1b\\`;
  184 | 
  185 |       await ctx.proxy.write(seq1);
  186 |       await timeout(50);
  187 |       strictEqual(await getImageStorageLength(), 0);
  188 | 
  189 |       await ctx.proxy.write(seq2);
  190 |       await timeout(100);
  191 |       strictEqual(await getImageStorageLength(), 1);
  192 |     });
  193 | 
  194 |     test('verifies chunked data is assembled correctly', async () => {
  195 |       const half = Math.floor(KITTY_BLACK_1X1_BASE64.length / 2);
  196 |       const part1 = KITTY_BLACK_1X1_BASE64.substring(0, half);
  197 |       const part2 = KITTY_BLACK_1X1_BASE64.substring(half);
  198 | 
  199 |       await ctx.proxy.write(`\x1b_Ga=t,f=100,i=99,m=1;${part1}\x1b\\`);
  200 |       await ctx.proxy.write(`\x1b_Ga=t,f=100,i=99;${part2}\x1b\\`);
  201 |       await timeout(100);
  202 | 
  203 |       const storedData = await ctx.page.evaluate(async () => {
  204 |         const blob = (window as any).imageAddon._handlers.get('kitty').images.get(99).data;
  205 |         const buffer = await blob.arrayBuffer();
  206 |         return Array.from(new Uint8Array(buffer));
  207 |       });
  208 |       deepStrictEqual(storedData, KITTY_BLACK_1X1_BYTES);
  209 |     });
  210 | 
  211 |     test('enforces size limit across chunked transmissions', async () => {
  212 |       // Create a custom addon with very small size limit (100 bytes)
```