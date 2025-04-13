## JS_TS_bun

A REPL-capable (not enabled by default) Typescript / Javascript interpreter.

`bun` needs to be installed and on your path.

### JS_TS_bun is REPL-capable

But the REPL is VERY quirky (and even has a greeting saying it's unstable)
It doesn't play well at all with sniprun's stdin-stdout
mechanism, so while basic examples are working,
I can't consider this a 'daily driver'... so REPL is disabled by default

```lua
require('sniprun').setup({
    selected_interpreters={"JS_TS_bun"},
    repl_enable={"JS_TS_bun"}
})
```

### more option for the (non-repl) command line

sniprun runs your code snippets with

`bun run --silent <file.ts>`

more arguments for `bun run` can be added with the interpreter option:

```lua
require'sniprun'.setup({
    interpreter_options = {
        JS_TS_bun = {
             bun_run_opts = "--smol"
            }
        }
    }
})
```
