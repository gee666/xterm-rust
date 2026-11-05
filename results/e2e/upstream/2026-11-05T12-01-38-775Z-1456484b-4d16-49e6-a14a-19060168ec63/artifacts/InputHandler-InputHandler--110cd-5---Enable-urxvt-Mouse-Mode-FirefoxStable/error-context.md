# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: InputHandler.test.ts >> InputHandler Integration Tests >> CSI >> CSI ? Pm h - DECSET: Private Mode Set >> Ps = 1 0 1 5 - Enable urxvt Mouse Mode
- Location: test\playwright\InputHandler.test.ts:604:11

# Error details

```
Error: page.evaluate: can't access property "activeEncoding", core.mouseStateService is undefined
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