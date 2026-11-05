# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: InputHandler.test.ts >> InputHandler Integration Tests >> CSI >> CSI ! p - DECSTR: Soft terminal reset, VT220 and up.
- Location: test\playwright\InputHandler.test.ts:1464:9

# Error details

```
Error: page.evaluate: TypeError: Cannot read properties of undefined (reading 'buffer')
    at eval (eval at evaluate (:302:30), <anonymous>:1:42)
    at eval (<anonymous>)
    at UtilityScript.evaluate (<anonymous>:302:30)
    at UtilityScript.<anonymous> (<anonymous>:1:44)
```

# Page snapshot

```yaml
- textbox "Terminal input" [ref=e6]
```