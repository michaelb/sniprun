## Rust original

a custom compiler can be specified :

```lua
require'sniprun'.setup({
    interpreter_options = {
        Rust_original = {
             compiler = "rustc"
             }
        }
    }
})
```

the Rust interpreter supports REPL mode via `evcxr` (needs to be installed)

```lua
require('sniprun').setup({
    repl_enable = { 'Rust_original' },
})
```
