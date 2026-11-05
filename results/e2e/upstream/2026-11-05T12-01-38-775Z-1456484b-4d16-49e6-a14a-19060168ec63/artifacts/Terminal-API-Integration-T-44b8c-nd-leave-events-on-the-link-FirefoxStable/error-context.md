# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: Terminal.test.ts >> API Integration Tests >> registerLinkProvider >> should fire hover and leave events on the link
- Location: test\playwright\Terminal.test.ts:860:9

# Error details

```
Error: page.evaluate: window.term.registerLinkProvider is not a function
@debugger eval code line 302 > eval:2:41
evaluate@debugger eval code:302:30
@debugger eval code:1:44
@debugger eval code:1:62

```

# Page snapshot

```yaml
- generic [ref=e5]:
  - textbox "Terminal input" [active] [ref=e6]
  - generic:
    - generic:
      - generic: foo bar baz
```