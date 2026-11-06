# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: WebglCustomGlyphs.test.ts >> WebGL custom glyphs >> pattern glyphs render after the terminal is adopted into another document
- Location: addons\addon-webgl\test\WebglCustomGlyphs.test.ts:32:7

# Error details

```
Error: page.evaluate: TypeError: window.WebglAddon is not a constructor
    at eval (eval at evaluate (:302:30), <anonymous>:1:16)
    at eval (<anonymous>)
    at UtilityScript.evaluate (<anonymous>:302:30)
    at UtilityScript.<anonymous> (<anonymous>:1:44)
```