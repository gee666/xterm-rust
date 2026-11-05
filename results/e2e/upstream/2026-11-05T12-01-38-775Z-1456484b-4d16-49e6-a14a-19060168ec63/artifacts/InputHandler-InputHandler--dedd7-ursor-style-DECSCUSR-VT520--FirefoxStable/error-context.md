# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: InputHandler.test.ts >> InputHandler Integration Tests >> CSI >> CSI Ps SP q - Set cursor style (DECSCUSR), VT520.
- Location: test\playwright\InputHandler.test.ts:1514:9

# Error details

```
Error: page.evaluate: can't access property "decPrivateModes", core.coreService is undefined
@debugger eval code line 302 > eval:2:23
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