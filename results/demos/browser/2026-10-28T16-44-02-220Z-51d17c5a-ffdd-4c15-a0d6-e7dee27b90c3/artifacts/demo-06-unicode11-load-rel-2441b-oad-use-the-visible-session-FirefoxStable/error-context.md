# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: demo-06.spec.cjs >> unicode11 load, reload, behavior and unload use the visible session
- Location: scripts\demo-e2e\demo-06.spec.cjs:7:28

# Error details

```
Error: page.evaluate: can't access property "activeVersion", demo.terminal.unicode is undefined
@debugger eval code line 302 > eval:1:7
evaluate@debugger eval code:304:16
@debugger eval code:1:44
@debugger eval code:1:62

```

# Page snapshot

```yaml
- generic [active] [ref=e1]:
  - banner [ref=e2]:
    - heading "xterm Rust demo" [level=1] [ref=e3]
    - paragraph [ref=e4]: Rust parser, buffers and rendering. Full xterm.js equivalence is not proven.
  - status [ref=e5]: Rust terminal ready
  - main [ref=e6]:
    - region "Terminal session" [ref=e7]:
      - generic [ref=e8]:
        - button "ANSI and Unicode sample" [ref=e9] [cursor=pointer]
        - button "Reset" [ref=e10] [cursor=pointer]
        - button "Dispose" [ref=e11] [cursor=pointer]
        - button "Reopen" [ref=e12] [cursor=pointer]
      - generic [ref=e14]:
        - generic [ref=e15]:
          - text: Incoming display text, or a JSON string with escapes
          - textbox "Incoming display text, or a JSON string with escapes" [ref=e16]: hello from Rust
        - button "Write" [ref=e17] [cursor=pointer]
      - generic [ref=e18]:
        - generic [ref=e19]:
          - text: Outgoing user input
          - textbox "Outgoing user input" [ref=e20]: user input
        - button "Send input" [ref=e21] [cursor=pointer]
      - group [ref=e22]:
        - generic "Live terminal state and outgoing data" [ref=e23] [cursor=pointer]
        - generic [ref=e24]: Terminal disposed
    - complementary "Live terminal controls" [ref=e25]:
      - group [ref=e26]:
        - generic "Size, style and renderer" [ref=e27] [cursor=pointer]
        - generic [ref=e28]:
          - generic [ref=e29]:
            - text: Size, COLSxROWS or auto
            - textbox "Size, COLSxROWS or auto" [ref=e30]: 80x24
          - button "Resize" [ref=e31] [cursor=pointer]
        - generic [ref=e32]:
          - text: Padding
          - spinbutton "Padding" [ref=e33]: "8"
        - generic [ref=e34]:
          - text: Theme
          - combobox "Theme" [ref=e35]:
            - option "default" [selected]
            - option "xtermjs"
            - option "sapphire"
            - option "light"
        - generic [ref=e36]:
          - text: Renderer
          - combobox "Renderer" [ref=e37]:
            - option "DOM" [selected]
            - option "WebGL"
      - group [ref=e38]:
        - generic "Options on this terminal" [ref=e39] [cursor=pointer]
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
      - group [ref=e40]:
        - generic "Search" [ref=e41] [cursor=pointer]
        - generic [ref=e42]:
          - text: Text
          - textbox "Text" [ref=e43]: Rust
        - generic [ref=e44]:
          - checkbox "Regex" [ref=e45]
          - text: Regex
        - generic [ref=e46]:
          - checkbox "Case sensitive" [ref=e47]
          - text: Case sensitive
        - generic [ref=e48]:
          - checkbox "Whole word" [ref=e49]
          - text: Whole word
        - generic [ref=e50]:
          - checkbox "Highlight all matches" [checked] [ref=e51]
          - text: Highlight all matches
        - button "Find next" [ref=e52] [cursor=pointer]
        - button "Find previous" [ref=e53] [cursor=pointer]
        - status
      - group [ref=e54]:
        - generic "Serialize" [ref=e55] [cursor=pointer]
        - button "Serialize ANSI" [ref=e56] [cursor=pointer]
        - generic [ref=e57]:
          - checkbox "Reset and write back to this terminal" [ref=e58]
          - text: Reset and write back to this terminal
        - button "Serialize HTML and copy" [ref=e59] [cursor=pointer]
      - group [ref=e60]:
        - generic "Addon lifecycle" [ref=e61] [cursor=pointer]
        - generic [ref=e62]:
          - text: Addon
          - combobox "Addon" [ref=e63]:
            - option "attach" [selected]
            - option "clipboard"
            - option "fit"
            - option "image"
            - option "ligatures"
            - option "program-status"
            - option "progress"
            - option "search"
            - option "serialize"
            - option "unicode-graphemes"
            - option "unicode11"
            - option "web-fonts"
            - option "web-links"
            - option "webgl"
        - generic [ref=e64]:
          - text: Attach WebSocket URL
          - textbox "Attach WebSocket URL" [ref=e65]: ws://127.0.0.1:3000/socket
        - button "Load" [ref=e66] [cursor=pointer]
        - button "Unload" [ref=e67] [cursor=pointer]
        - button "Reload" [ref=e68] [cursor=pointer]
        - generic [ref=e69]: "[]"
  - group [ref=e70]:
    - generic "Action and lifecycle log" [ref=e71] [cursor=pointer]
```