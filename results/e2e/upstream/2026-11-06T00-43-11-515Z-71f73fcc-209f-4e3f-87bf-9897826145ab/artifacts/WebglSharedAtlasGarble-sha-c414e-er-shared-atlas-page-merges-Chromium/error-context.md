# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: WebglSharedAtlasGarble.test.ts >> shared-atlas garble across terminals (#6038) >> keeps a second terminal rendering correctly after shared atlas page merges
- Location: addons\addon-webgl\test\WebglSharedAtlasGarble.test.ts:209:7

# Error details

```
Error: page.evaluate: TypeError: window.WebglAddon is not a constructor
    at eval (eval at evaluate (:302:30), <anonymous>:2:20)
    at UtilityScript.evaluate (<anonymous>:304:16)
    at UtilityScript.<anonymous> (<anonymous>:1:44)
```