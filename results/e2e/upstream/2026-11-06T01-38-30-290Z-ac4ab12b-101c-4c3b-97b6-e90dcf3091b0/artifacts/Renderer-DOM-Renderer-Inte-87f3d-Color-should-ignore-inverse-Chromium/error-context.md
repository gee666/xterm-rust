# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: Renderer.test.ts >> DOM Renderer Integration Tests >> decoration color overrides >> foregroundColor should ignore inverse
- Location: test\playwright\SharedRendererTests.ts:1090:9

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
- generic [ref=e5]:
  - generic:
    - textbox "Terminal input"
```