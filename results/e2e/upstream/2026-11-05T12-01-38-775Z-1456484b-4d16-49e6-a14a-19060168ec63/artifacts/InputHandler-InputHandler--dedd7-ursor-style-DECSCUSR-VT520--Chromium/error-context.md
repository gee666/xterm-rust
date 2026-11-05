# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: InputHandler.test.ts >> InputHandler Integration Tests >> CSI >> CSI Ps SP q - Set cursor style (DECSCUSR), VT520.
- Location: test\playwright\InputHandler.test.ts:1514:9

# Error details

```
Error: page.evaluate: TypeError: Cannot read properties of undefined (reading 'decPrivateModes')
    at eval (eval at evaluate (:302:30), <anonymous>:2:40)
    at UtilityScript.evaluate (<anonymous>:304:16)
    at UtilityScript.<anonymous> (<anonymous>:1:44)
```

# Page snapshot

```yaml
- generic [ref=e5]:
  - generic:
    - textbox "Terminal input"
```