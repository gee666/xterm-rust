# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: WebglRenderer.test.ts >> WebGL Renderer Integration Tests >> decoration color overrides >> foregroundColor
- Location: test\playwright\SharedRendererTests.ts:1077:9

# Error details

```
Error: page.evaluate: TypeError: window.term.registerDecoration is not a function
    at eval (eval at evaluate (:302:30), <anonymous>:2:21)
    at eval (<anonymous>)
    at UtilityScript.evaluate (<anonymous>:302:30)
    at UtilityScript.<anonymous> (<anonymous>:1:44)
```

# Page snapshot

```yaml
- textbox "Terminal input" [ref=e6]
```