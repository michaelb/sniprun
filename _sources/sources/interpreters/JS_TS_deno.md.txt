## JS_TS_Deno

A REPL-capable (not enabled by default) Typescript / Javascript interpreter.

`deno` needs to be installed and on your path (and working).
The precise command used by sniprun is `deno repl -q`

```lua
require('sniprun').setup({
    selected_interpreters={"JS_TS_deno"},
    repl_enable={"JS_TS_deno"}
})
```
