# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: KittyGraphics.test.ts >> Kitty Graphics Protocol >> Raw RGB pixel format (f=24) >> Pixel verification >> renders 3x1 strip (red, green, blue)
- Location: addons\addon-image\test\KittyGraphics.test.ts:1848:11

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