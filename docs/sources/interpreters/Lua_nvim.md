## Lua_nvim

Limitations:

This interpreter works inherently in a pseudo-REPL mode and this can't be disabled.

Essentially, you can expect REPL behavior when running line-by-line of bloc-by-bloc lua script:

```
a = 4 
b = 6
print(a+5) -- <- 9

a = 0

print(a + b) -- <- 6
```

HOWEVER, if you define a 'local' variable, it won't be available in subsequent calls

```
local a = 5

print(a) -- <- nil
```

