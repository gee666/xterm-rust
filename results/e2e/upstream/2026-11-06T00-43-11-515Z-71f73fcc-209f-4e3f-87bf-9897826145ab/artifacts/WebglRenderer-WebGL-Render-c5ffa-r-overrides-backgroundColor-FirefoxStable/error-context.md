# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: WebglRenderer.test.ts >> WebGL Renderer Integration Tests >> decoration color overrides >> backgroundColor
- Location: test\playwright\SharedRendererTests.ts:1117:9

# Error details

```
Error: page.evaluate: window.term.registerDecoration is not a function
@debugger eval code line 302 > eval:2:21
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