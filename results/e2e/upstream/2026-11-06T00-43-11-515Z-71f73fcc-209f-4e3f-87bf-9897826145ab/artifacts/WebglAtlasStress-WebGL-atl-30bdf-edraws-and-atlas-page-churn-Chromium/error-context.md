# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: WebglAtlasStress.test.ts >> WebGL atlas TUI stress (#6038) >> primary-buffer glyphs survive alternate-buffer redraws and atlas page churn
- Location: addons\addon-webgl\test\WebglAtlasStress.test.ts:243:7

# Error details

```
Error: page.evaluate: TypeError: w.WebglAddon is not a constructor
    at eval (eval at evaluate (:302:30), <anonymous>:3:15)
    at UtilityScript.evaluate (<anonymous>:304:16)
    at UtilityScript.<anonymous> (<anonymous>:1:44)
```