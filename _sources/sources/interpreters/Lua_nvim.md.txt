## Lua_nvim

This interpreter works inherently in a pseudo-REPL mode and this can't
be disabled. However, it is run within neovim so you can expect the
usual vim API functions to be available.

Essentially, you can expect REPL behavior when running line-by-line
of bloc-by-bloc lua script:

```lua
a = 4 
b = 6
print(a+5) -- <- 9

a = 0

print(a + b) -- <- 6
```
