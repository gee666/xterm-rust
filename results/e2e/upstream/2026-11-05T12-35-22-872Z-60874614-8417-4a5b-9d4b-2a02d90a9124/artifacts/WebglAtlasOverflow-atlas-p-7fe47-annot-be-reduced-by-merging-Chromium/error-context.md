# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: WebglAtlasOverflow.test.ts >> atlas page overflow (#6038) >> evicts normal pages when the page cap cannot be reduced by merging
- Location: addons\addon-webgl\test\WebglAtlasOverflow.test.ts:331:7

# Error details

```
Error: page.evaluate: TypeError: w.WebglAddon is not a constructor
    at eval (eval at evaluate (:302:30), <anonymous>:3:15)
    at UtilityScript.evaluate (<anonymous>:304:16)
    at UtilityScript.<anonymous> (<anonymous>:1:44)
```