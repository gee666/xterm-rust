# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: InputHandler.test.ts >> InputHandler Integration Tests >> CSI >> CSI ? Pm h - DECSET: Private Mode Set >> Ps = 1 0 4 1 - Use the CLIPBOARD selection, xterm
- Location: test\playwright\InputHandler.test.ts:632:11

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