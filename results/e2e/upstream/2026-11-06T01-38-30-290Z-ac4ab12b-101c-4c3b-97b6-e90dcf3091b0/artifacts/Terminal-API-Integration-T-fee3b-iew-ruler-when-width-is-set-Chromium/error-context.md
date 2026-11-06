# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: Terminal.test.ts >> API Integration Tests >> registerDecoration >> overviewRulerDecorations >> should add an overview ruler when width is set
- Location: test\playwright\Terminal.test.ts:826:11

# Error details

```
Error: page.evaluate: TypeError: window.term.registerDecoration is not a function
    at eval (eval at evaluate (:302:30), <anonymous>:1:13)
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