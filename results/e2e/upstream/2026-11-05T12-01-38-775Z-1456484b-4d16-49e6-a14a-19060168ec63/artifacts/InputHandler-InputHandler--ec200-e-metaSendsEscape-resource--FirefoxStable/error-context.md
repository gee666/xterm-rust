# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: InputHandler.test.ts >> InputHandler Integration Tests >> CSI >> CSI ? Pm l - DECRST: DEC Private Mode Reset >> Ps = 1 0 3 6 - Don't send ESC  when Meta modifies a key, xterm.  This disables the metaSendsEscape resource.
- Location: test\playwright\InputHandler.test.ts:956:11

# Error details

```
Error: page.evaluate: can't access property "activeProtocol", core.mouseStateService is undefined
@debugger eval code line 302 > eval:1:13
evaluate@debugger eval code:304:16
@debugger eval code:1:44
@debugger eval code:1:62

```

# Page snapshot

```yaml
- generic [ref=e5]:
  - generic:
    - textbox "Terminal input"
```