# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: Terminal.test.ts >> API Integration Tests >> registerDecoration >> bufferDecorations >> should return undefined when the marker has already been disposed of
- Location: test\playwright\Terminal.test.ts:798:11

# Error details

```
Error: page.evaluate: window.term.registerDecoration is not a function
@debugger eval code line 302 > eval:1:33
evaluate@debugger eval code:302:30
@debugger eval code:1:44
@debugger eval code:1:62

```

# Page snapshot

```yaml
- generic [ref=e5]:
  - generic:
    - textbox "Terminal input"
```