# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: WebglRenderer.test.ts >> WebGL Renderer Integration Tests >> decoration color overrides >> backgroundColor should ignore inverse (only bg on decoration)
- Location: test\playwright\SharedRendererTests.ts:1143:9

# Error details

```
Error: page.evaluate: TypeError: window.term.registerDecoration is not a function. (In 'window.term.registerDecoration({
          marker,
          width: 2,
          backgroundColor: '#0000ff'
        })', 'window.term.registerDecoration' is undefined)
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
```

# Test source

```ts
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
  1078 |       await ctx.value.page.evaluate(`
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
> 1146 |       await ctx.value.page.evaluate(`
       |                            ^ Error: page.evaluate: TypeError: window.term.registerDecoration is not a function. (In 'window.term.registerDecoration({
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
  1179 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 2, 1), [0, 0, 255, 255]);
  1180 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 2, 2), [0, 0, 255, 255]);
  1181 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 2, 3), [0, 0, 255, 255]);
  1182 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 3, 3), [0, 0, 255, 255]);
  1183 |     });
  1184 |     // due to #5852
  1185 |     test.skip('#4758: multiple invisible text characters without SGR change should not be rendered', async () => {
  1186 |       // Regression test: #4758 when multiple invisible characters are used
  1187 |       await ctx.value.proxy.writeln(`■\x1b[8m■■`);
  1188 |       // Full refresh as the before result is the same as after
  1189 |       const rows = await ctx.value.proxy.rows;
  1190 |       await waitForRenderAfter(ctx.value, () => ctx.value.proxy.refresh(0, rows - 1));
  1191 |       await pollFor(ctx.value.page, () => getCellDomChar(ctx.value, 1, 1), '■');
  1192 |       await pollFor(ctx.value.page, () => getCellDomChar(ctx.value, 2, 1), ' ');
  1193 |       await pollFor(ctx.value.page, () => getCellDomChar(ctx.value, 3, 1), ' ');
  1194 |     });
  1195 |     // HACK: It's not clear why DOM is failing here
  1196 |     (ctx.skipDomExceptions ? test.skip : test)('#4759: minimum contrast ratio should be respected on inverse text', async () => {
  1197 |       const theme: ITheme = {
  1198 |         foreground: '#aaaaaa',
  1199 |         background: '#333333'
  1200 |       };
  1201 |       await ctx.value.page.evaluate(`window.term.options.theme = ${JSON.stringify(theme)};`);
  1202 |       await ctx.value.proxy.write(`\x1b[7m■■`);
  1203 |       // Validate before minimumContrastRatio is applied
  1204 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [51, 51, 51, 255]);
  1205 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 2, 1), [51, 51, 51, 255]);
  1206 |       await ctx.value.page.evaluate(`window.term.options.minimumContrastRatio = 10;`);
  1207 |       frameDetails = undefined;
  1208 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [0, 0, 0, 255]);
  1209 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 2, 1), [0, 0, 0, 255]);
  1210 |     });
  1211 |     test('#4759: minimum contrast ratio should be respected on selected inverse text', async () => {
  1212 |       const theme: ITheme = {
  1213 |         foreground: '#777777',
  1214 |         background: '#555555',
  1215 |         selectionBackground: '#666666' // Slightly more contrast needed for selection
  1216 |       };
  1217 |       await ctx.value.page.evaluate(`window.term.options.theme = ${JSON.stringify(theme)};`);
  1218 |       await ctx.value.proxy.write(`\x1b[7m■■`);
  1219 |       await ctx.value.proxy.selectAll();
  1220 |       // Validate before minimumContrastRatio is applied
  1221 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [85, 85, 85, 255]);
  1222 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 2, 1), [85, 85, 85, 255]);
  1223 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 3, 1), [102, 102, 102, 255]);
  1224 |       await ctx.value.page.evaluate(`window.term.options.minimumContrastRatio = 10;`);
  1225 |       await ctx.value.proxy.selectAll();
  1226 |       frameDetails = undefined;
  1227 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [255, 255, 255, 255]);
  1228 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 2, 1), [255, 255, 255, 255]);
  1229 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 3, 1), [102, 102, 102, 255]);
  1230 |     });
  1231 |     test('#4773: block cursor should render when the cell is selected', async () => {
  1232 |       const theme: ITheme = {
  1233 |         cursor: '#0000FF',
  1234 |         selectionBackground: '#FF0000'
  1235 |       };
  1236 |       await ctx.value.page.evaluate(`window.term.options.theme = ${JSON.stringify(theme)};`);
  1237 |       await ctx.value.proxy.focus();
  1238 |       await ctx.value.proxy.selectAll();
  1239 |       await pollFor(ctx.value.page, () => getCellColor(ctx.value, 1, 1), [0, 0, 255, 255]);
  1240 |     });
  1241 |     test('#4799: cursor should be in the correct position', async () => {
  1242 |       const theme: ITheme = {
  1243 |         cursor: '#0000FF'
  1244 |       };
  1245 |       await ctx.value.page.evaluate(`window.term.options.theme = ${JSON.stringify(theme)};`);
  1246 |       for (let index = 0; index < 160; index++) {
```