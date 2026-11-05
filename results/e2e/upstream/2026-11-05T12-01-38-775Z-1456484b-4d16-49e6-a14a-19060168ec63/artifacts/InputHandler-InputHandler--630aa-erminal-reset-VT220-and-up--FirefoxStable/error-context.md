# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: InputHandler.test.ts >> InputHandler Integration Tests >> CSI >> CSI ! p - DECSTR: Soft terminal reset, VT220 and up.
- Location: test\playwright\InputHandler.test.ts:1464:9

# Error details

```
Error: page.evaluate: can't access property "buffer", window.term._core._bufferService is undefined
@debugger eval code line 302 > eval:1:4
evaluate@debugger eval code:302:30
@debugger eval code:1:44
@debugger eval code:1:62

```

# Page snapshot

```yaml
- textbox "Terminal input" [ref=e6]
```