# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: WebglRenderer.test.ts >> WebGL Renderer Integration Tests >> decoration color overrides >> foregroundColor
- Location: test\playwright\SharedRendererTests.ts:1077:9

# Error details

```
Error: page.evaluate: TypeError: window.term.registerDecoration is not a function. (In 'window.term.registerDecoration({
          marker,
          foregroundColor: '#ff0000',
          backgroundColor: '#0000ff'
        })', 'window.term.registerDecoration' is undefined)
```

# Page snapshot

```yaml
- generic [ref=e6]:
  - textbox "Terminal input"
  - text: WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW
```

# Test source

```ts
  978  |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 3, 1), [0, 255, 0, 255]);
  979  |     });
  980  |   });
  981  | 
  982  |   test.describe('selectionInactiveBackground', async () => {
  983  |     test('should render the inactive selection when not focused', async () => {
  984  |       const theme: ITheme = {
  985  |         selectionBackground: '#FF000080',
  986  |         selectionInactiveBackground: '#0000FF80'
  987  |       };
  988  |       await ctx.value.page.evaluate(`window.term.options.theme = ${JSON.stringify(theme)};`);
  989  |       await ctx.value.proxy.focus();
  990  |       // Check both the cursor line and another line
  991  |       await ctx.value.proxy.writeln('_ ');
  992  |       await ctx.value.proxy.write('_ ');
  993  |       await ctx.value.proxy.selectAll();
  994  |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [128, 0, 0, 255]);
  995  |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 2, 1), [128, 0, 0, 255]);
  996  |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 2), [128, 0, 0, 255]);
  997  |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 2, 2), [128, 0, 0, 255]);
  998  |       await ctx.value.page.evaluate(`document.activeElement.blur()`);
  999  |       frameDetails = undefined;
  1000 |       // Selection only cell needs to be first to ensure renderer has kicked in
  1001 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [0, 0, 128, 255]);
  1002 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 2, 1), [0, 0, 128, 255]);
  1003 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 2), [0, 0, 128, 255]);
  1004 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 2, 2), [0, 0, 128, 255]);
  1005 |     });
  1006 |   });
  1007 | 
  1008 |   ctx.skipDomExceptions ? test.describe.skip : test.describe('selection blending', () => {
  1009 |     test('background', async () => {
  1010 |       const theme: ITheme = {
  1011 |         red: '#CC0000',
  1012 |         selectionBackground: '#FFFFFF'
  1013 |       };
  1014 |       await ctx.value.page.evaluate(`window.term.options.theme = ${JSON.stringify(theme)};`);
  1015 |       await ctx.value.proxy.focus();
  1016 |       await ctx.value.proxy.writeln('\x1b[41m red bg\x1b[0m');
  1017 |       await ctx.value.proxy.writeln('\x1b[7m inverse\x1b[0m');
  1018 |       await ctx.value.proxy.writeln('\x1b[31;7m red fg inverse\x1b[0m');
  1019 |       await ctx.value.proxy.writeln('\x1b[48:2:0:204:0:0m red truecolor bg\x1b[0m');
  1020 |       await ctx.value.proxy.selectAll();
  1021 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [230, 128, 128, 255]);
  1022 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 2), [255, 255, 255, 255]);
  1023 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 3), [230, 128, 128, 255]);
  1024 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 4), [230, 128, 128, 255]);
  1025 |     });
  1026 |     test('powerline decorative symbols', async () => {
  1027 |       const theme: ITheme = {
  1028 |         red: '#CC0000',
  1029 |         green: '#00CC00',
  1030 |         selectionBackground: '#FFFFFF'
  1031 |       };
  1032 |       await ctx.value.page.evaluate(`window.term.options.theme = ${JSON.stringify(theme)};`);
  1033 |       await ctx.value.proxy.focus();
  1034 |       await ctx.value.proxy.writeln('\u{E0B4} plain\x1b[0m');
  1035 |       await ctx.value.proxy.writeln('\x1b[31;42m\u{E0B4} red fg green bg\x1b[0m');
  1036 |       await ctx.value.proxy.writeln('\x1b[32;41m\u{E0B4} green fg red bg\x1b[0m');
  1037 |       await ctx.value.proxy.writeln('\x1b[31;42;7m\u{E0B4} red fg green bg inverse\x1b[0m');
  1038 |       await ctx.value.proxy.writeln('\x1b[32;41;7m\u{E0B4} green fg red bg inverse\x1b[0m');
  1039 |       await ctx.value.proxy.selectAll();
  1040 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [255, 255, 255, 255]);
  1041 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 2), [230, 128, 128, 255]);
  1042 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 3), [128, 230, 128, 255]);
  1043 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 4), [128, 230, 128, 255]);
  1044 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 5), [230, 128, 128, 255]);
  1045 |     });
  1046 |   });
  1047 | 
  1048 |   test.describe('allowTransparency', async () => {
  1049 |     test.beforeEach(() => ctx.value.page.evaluate(`term.options.allowTransparency = true`));
  1050 | 
  1051 |     test('transparent background inverse', async () => {
  1052 |       const theme: ITheme = {
  1053 |         background: '#ff000080'
  1054 |       };
  1055 |       await ctx.value.page.evaluate(`window.term.options.theme = ${JSON.stringify(theme)};`);
  1056 |       const data = `\x1b[7m■\x1b[0m`;
  1057 |       await ctx.value.proxy.write( data);
  1058 |       // Inverse background should be opaque
  1059 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [255, 0, 0, 255]);
  1060 |     });
  1061 |   });
  1062 | 
  1063 |   test.describe('selectionForeground', () => {
  1064 |     test('transparent background inverse', async () => {
  1065 |       const theme: ITheme = {
  1066 |         selectionForeground: '#ff0000'
  1067 |       };
  1068 |       await ctx.value.page.evaluate(`window.term.options.theme = ${JSON.stringify(theme)};`);
  1069 |       const data = `\x1b[7m■\x1b[0m`;
  1070 |       await ctx.value.proxy.write( data);
  1071 |       await ctx.value.proxy.selectAll();
  1072 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [255, 0, 0, 255]);
  1073 |     });
  1074 |   });
  1075 | 
  1076 |   test.describe('decoration color overrides', async () => {
  1077 |     test('foregroundColor', async () => {
> 1078 |       await ctx.value.page.evaluate(`
       |                            ^ Error: page.evaluate: TypeError: window.term.registerDecoration is not a function. (In 'window.term.registerDecoration({
  1079 |         const marker = window.term.registerMarker(-window.term.buffer.active.cursorY);
  1080 |         window.term.registerDecoration({
  1081 |           marker,
  1082 |           foregroundColor: '#ff0000',
  1083 |           backgroundColor: '#0000ff'
  1084 |         });
  1085 |       `);
  1086 |       const data = `■`;
  1087 |       await ctx.value.proxy.write( data);
  1088 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [255, 0, 0, 255]);
  1089 |     });
  1090 |     test('foregroundColor should ignore inverse', async () => {
  1091 |       await ctx.value.page.evaluate(`
  1092 |         const marker = window.term.registerMarker(-window.term.buffer.active.cursorY);
  1093 |         window.term.registerDecoration({
  1094 |           marker,
  1095 |           foregroundColor: '#ff0000',
  1096 |           backgroundColor: '#0000ff'
  1097 |         });
  1098 |       `);
  1099 |       const data = `\x1b[7m■\x1b[0m`;
  1100 |       await ctx.value.proxy.write( data);
  1101 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [255, 0, 0, 255]);
  1102 |     });
  1103 |     test('foregroundColor should ignore inverse (only fg on decoration)', async () => {
  1104 |       await ctx.value.page.evaluate(`
  1105 |         const marker = window.term.registerMarker(-window.term.buffer.active.cursorY);
  1106 |         window.term.registerDecoration({
  1107 |           marker,
  1108 |           width: 2,
  1109 |           foregroundColor: '#ff0000'
  1110 |         });
  1111 |       `);
  1112 |       const data = `\x1b[7m■ \x1b[0m`;
  1113 |       await ctx.value.proxy.write( data);
  1114 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [255, 0, 0, 255]); // inverse foreground of '■' should be decoration fg override
  1115 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 2, 1), [255, 255, 255, 255]); // inverse background of ' ' should be default foreground
  1116 |     });
  1117 |     test('backgroundColor', async () => {
  1118 |       await ctx.value.page.evaluate(`
  1119 |         const marker = window.term.registerMarker(-window.term.buffer.active.cursorY);
  1120 |         window.term.registerDecoration({
  1121 |           marker,
  1122 |           foregroundColor: '#ff0000',
  1123 |           backgroundColor: '#0000ff'
  1124 |         });
  1125 |       `);
  1126 |       const data = ` `;
  1127 |       await ctx.value.proxy.write( data);
  1128 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [0, 0, 255, 255]);
  1129 |     });
  1130 |     test('backgroundColor should ignore inverse', async () => {
  1131 |       await ctx.value.page.evaluate(`
  1132 |         const marker = window.term.registerMarker(-window.term.buffer.active.cursorY);
  1133 |         window.term.registerDecoration({
  1134 |           marker,
  1135 |           foregroundColor: '#ff0000',
  1136 |           backgroundColor: '#0000ff'
  1137 |         });
  1138 |       `);
  1139 |       const data = `\x1b[7m \x1b[0m`;
  1140 |       await ctx.value.proxy.write( data);
  1141 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [0, 0, 255, 255]);
  1142 |     });
  1143 |     test('backgroundColor should ignore inverse (only bg on decoration)', async () => {
  1144 |       const data = `\x1b[7m■ \x1b[0m`;
  1145 |       await ctx.value.proxy.write( data);
  1146 |       await ctx.value.page.evaluate(`
  1147 |         const marker = window.term.registerMarker(-window.term.buffer.active.cursorY);
  1148 |         window.term.registerDecoration({
  1149 |           marker,
  1150 |           width: 2,
  1151 |           backgroundColor: '#0000ff'
  1152 |         });
  1153 |       `);
  1154 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [0, 0, 0, 255]); // inverse foreground of '■' should be default
  1155 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 2, 1), [0, 0, 255, 255]); // inverse background of ' ' should be decoration bg override
  1156 |     });
  1157 |   });
  1158 | 
  1159 |   test.describe('regression tests', () => {
  1160 |     test('#4736: inactive selection background should replace regular cell background color', async () => {
  1161 |       const theme: ITheme = {
  1162 |         selectionBackground: '#FF0000',
  1163 |         selectionInactiveBackground: '#0000FF'
  1164 |       };
  1165 |       await ctx.value.page.evaluate(`window.term.options.theme = ${JSON.stringify(theme)};`);
  1166 |       await ctx.value.proxy.writeln(' ');
  1167 |       await ctx.value.proxy.writeln(' O ');
  1168 |       await ctx.value.proxy.writeln(' ');
  1169 |       await ctx.value.proxy.focus();
  1170 |       await ctx.value.proxy.selectAll();
  1171 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [255, 0, 0, 255]);
  1172 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 2, 1), [255, 0, 0, 255]);
  1173 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 2, 2), [255, 0, 0, 255]);
  1174 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 2, 3), [255, 0, 0, 255]);
  1175 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 3, 3), [255, 0, 0, 255]);
  1176 |       await ctx.value.proxy.blur();
  1177 |       frameDetails = undefined;
  1178 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [0, 0, 255, 255]);
```