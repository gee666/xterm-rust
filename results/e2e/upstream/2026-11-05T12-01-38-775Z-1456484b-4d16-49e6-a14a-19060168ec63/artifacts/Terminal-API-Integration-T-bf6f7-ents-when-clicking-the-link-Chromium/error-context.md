# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: Terminal.test.ts >> API Integration Tests >> registerLinkProvider >> should fire activate events when clicking the link
- Location: test\playwright\Terminal.test.ts:941:9

# Error details

```
Error: page.evaluate: TypeError: window.term.registerLinkProvider is not a function
    at eval (eval at evaluate (:302:30), <anonymous>:2:41)
    at eval (<anonymous>)
    at UtilityScript.evaluate (<anonymous>:302:30)
    at UtilityScript.<anonymous> (<anonymous>:1:44)
```

# Page snapshot

```yaml
- generic [ref=e5]:
  - textbox "Terminal input" [active] [ref=e6]
  - generic:
    - generic:
      - generic: a b c
```