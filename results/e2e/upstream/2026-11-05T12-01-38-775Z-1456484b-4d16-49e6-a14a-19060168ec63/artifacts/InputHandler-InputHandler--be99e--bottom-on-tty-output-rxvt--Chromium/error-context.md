# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: InputHandler.test.ts >> InputHandler Integration Tests >> CSI >> CSI ? Pm h - DECSET: Private Mode Set >> Ps = 1 0 1 0 - Scroll to bottom on tty output (rxvt)
- Location: test\playwright\InputHandler.test.ts:598:11

# Error details

```
Error: page.evaluate: TypeError: Cannot read properties of undefined (reading 'activeProtocol')
    at eval (eval at evaluate (:302:30), <anonymous>:1:36)
    at UtilityScript.evaluate (<anonymous>:304:16)
    at UtilityScript.<anonymous> (<anonymous>:1:44)
```

# Page snapshot

```yaml
- generic [ref=e5]:
  - generic:
    - textbox "Terminal input"
```