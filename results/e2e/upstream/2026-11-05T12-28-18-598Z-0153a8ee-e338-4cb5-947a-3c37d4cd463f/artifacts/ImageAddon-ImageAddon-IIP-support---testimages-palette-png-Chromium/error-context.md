# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: ImageAddon.test.ts >> ImageAddon >> IIP support - testimages >> palette.png
- Location: addons\addon-image\test\ImageAddon.test.ts:355:9

# Error details

```
Error: page.evaluate: ReferenceError: ImageAddon is not defined
    at eval (eval at evaluate (:302:30), <anonymous>:3:31)
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