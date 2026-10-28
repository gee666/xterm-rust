# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: demo-06.spec.cjs >> ligatures load, reload, behavior and unload use the visible session
- Location: scripts\demo-e2e\demo-06.spec.cjs:7:28

# Error details

```
Error: page.evaluate: Error: addon-load: load@http://127.0.0.1:55764/demo/addons.mjs:31:59
action@http://127.0.0.1:55764/demo/app.mjs:53:34
action@http://127.0.0.1:55764/demo/app.mjs:110:54
@http://127.0.0.1:55764/demo/controls.mjs:48:17
```

# Page snapshot

```yaml
- generic [active] [ref=e1]:
  - banner [ref=e2]:
    - heading "xterm Rust demo" [level=1] [ref=e3]
    - paragraph [ref=e4]: Rust parser, buffers and rendering. Full xterm.js equivalence is not proven.
  - status [ref=e5]: Action failed. See the error and lifecycle log.
  - alert [ref=e6]: "addon-load: load@http://127.0.0.1:55764/demo/addons.mjs:31:59 action@http://127.0.0.1:55764/demo/app.mjs:53:34 action@http://127.0.0.1:55764/demo/app.mjs:110:54 @http://127.0.0.1:55764/demo/controls.mjs:48:17"
  - main [ref=e7]:
    - region "Terminal session" [ref=e8]:
      - generic [ref=e9]:
        - button "ANSI and Unicode sample" [ref=e10] [cursor=pointer]
        - button "Reset" [ref=e11] [cursor=pointer]
        - button "Dispose" [ref=e12] [cursor=pointer]
        - button "Reopen" [ref=e13] [cursor=pointer]
      - generic [ref=e15]:
        - generic [ref=e16]:
          - text: Incoming display text, or a JSON string with escapes
          - textbox "Incoming display text, or a JSON string with escapes" [ref=e17]: hello from Rust
        - button "Write" [ref=e18] [cursor=pointer]
      - generic [ref=e19]:
        - generic [ref=e20]:
          - text: Outgoing user input
          - textbox "Outgoing user input" [ref=e21]: user input
        - button "Send input" [ref=e22] [cursor=pointer]
      - group [ref=e23]:
        - generic "Live terminal state and outgoing data" [ref=e24] [cursor=pointer]
        - generic [ref=e25]: Terminal disposed
    - complementary "Live terminal controls" [ref=e26]:
      - group [ref=e27]:
        - generic "Size, style and renderer" [ref=e28] [cursor=pointer]
        - generic [ref=e29]:
          - generic [ref=e30]:
            - text: Size, COLSxROWS or auto
            - textbox "Size, COLSxROWS or auto" [ref=e31]: 80x24
          - button "Resize" [ref=e32] [cursor=pointer]
        - generic [ref=e33]:
          - text: Padding
          - spinbutton "Padding" [ref=e34]: "8"
        - generic [ref=e35]:
          - text: Theme
          - combobox "Theme" [ref=e36]:
            - option "default" [selected]
            - option "xtermjs"
            - option "sapphire"
            - option "light"
        - generic [ref=e37]:
          - text: Renderer
          - combobox "Renderer" [ref=e38]:
            - option "DOM" [selected]
            - option "WebGL"
      - group [ref=e39]:
        - generic "Options on this terminal" [ref=e40] [cursor=pointer]
        - option "block" [selected]
        - option "underline"
        - option "bar"
        - option "outline" [selected]
        - option "block"
        - option "bar"
        - option "underline"
        - option "none"
        - option "normal" [selected]
        - option "bold"
        - option "100"
        - option "200"
        - option "300"
        - option "400"
        - option "500"
        - option "600"
        - option "700"
        - option "800"
        - option "900"
        - option "normal"
        - option "bold" [selected]
        - option "100"
        - option "200"
        - option "300"
        - option "400"
        - option "500"
        - option "600"
        - option "700"
        - option "800"
        - option "900"
        - option "trace"
        - option "debug"
        - option "info" [selected]
        - option "warn"
        - option "error"
        - option "off"
      - group [ref=e41]:
        - generic "Search" [ref=e42] [cursor=pointer]
        - generic [ref=e43]:
          - text: Text
          - textbox "Text" [ref=e44]: Rust
        - generic [ref=e45]:
          - checkbox "Regex" [ref=e46]
          - text: Regex
        - generic [ref=e47]:
          - checkbox "Case sensitive" [ref=e48]
          - text: Case sensitive
        - generic [ref=e49]:
          - checkbox "Whole word" [ref=e50]
          - text: Whole word
        - generic [ref=e51]:
          - checkbox "Highlight all matches" [checked] [ref=e52]
          - text: Highlight all matches
        - button "Find next" [ref=e53] [cursor=pointer]
        - button "Find previous" [ref=e54] [cursor=pointer]
        - status
      - group [ref=e55]:
        - generic "Serialize" [ref=e56] [cursor=pointer]
        - button "Serialize ANSI" [ref=e57] [cursor=pointer]
        - generic [ref=e58]:
          - checkbox "Reset and write back to this terminal" [ref=e59]
          - text: Reset and write back to this terminal
        - button "Serialize HTML and copy" [ref=e60] [cursor=pointer]
      - group [ref=e61]:
        - generic "Addon lifecycle" [ref=e62] [cursor=pointer]
        - generic [ref=e63]:
          - text: Addon
          - combobox "Addon" [ref=e64]:
            - option "attach"
            - option "clipboard"
            - option "fit"
            - option "image"
            - option "ligatures" [selected]
            - option "program-status"
            - option "progress"
            - option "search"
            - option "serialize"
            - option "unicode-graphemes"
            - option "unicode11"
            - option "web-fonts"
            - option "web-links"
            - option "webgl"
        - generic [ref=e65]:
          - text: Attach WebSocket URL
          - textbox "Attach WebSocket URL" [ref=e66]: ws://127.0.0.1:55764/socket
        - button "Load" [ref=e67] [cursor=pointer]
        - button "Unload" [ref=e68] [cursor=pointer]
        - button "Reload" [ref=e69] [cursor=pointer]
        - generic [ref=e70]: "[]"
  - group [ref=e71]:
    - generic "Action and lifecycle log" [ref=e72] [cursor=pointer]
```