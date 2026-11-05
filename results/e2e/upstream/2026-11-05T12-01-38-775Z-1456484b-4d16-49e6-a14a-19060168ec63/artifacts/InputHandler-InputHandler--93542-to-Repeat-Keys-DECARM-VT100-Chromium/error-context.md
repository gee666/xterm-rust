# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: InputHandler.test.ts >> InputHandler Integration Tests >> CSI >> CSI ? Pm h - DECSET: Private Mode Set >> Ps = 8 - Auto-Repeat Keys (DECARM), VT100
- Location: test\playwright\InputHandler.test.ts:416:11

# Error details

```
Error: page.evaluate: TypeError: Cannot read properties of undefined (reading 'activeProtocol')
    at eval (eval at evaluate (:302:30), <anonymous>:1:36)
    at UtilityScript.evaluate (<anonymous>:304:16)
    at UtilityScript.<anonymous> (<anonymous>:1:44)
```

# Page snapshot

```yaml
- textbox "Terminal input" [ref=e6]
```