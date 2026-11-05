# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: Terminal.test.ts >> API Integration Tests >> dispose (opened)
- Location: test\playwright\Terminal.test.ts:760:7

# Error details

```
Error: page.evaluate: TypeError: Cannot read properties of undefined (reading '_isDisposed')
    at eval (eval at evaluate (:302:30), <anonymous>:1:26)
    at eval (<anonymous>)
    at UtilityScript.evaluate (<anonymous>:302:30)
    at UtilityScript.<anonymous> (<anonymous>:1:44)
```