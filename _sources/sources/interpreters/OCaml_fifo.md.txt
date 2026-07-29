## OCaml fifo

This interpreter relies on `ocaml` by default, but has been
confirmed to work with  `utop` in normal (non-REPL) mode.

The default interpreter can be changed relatively safely for non-REPL mode:

```lua
require'sniprun'.setup({
    interpreter_options = {
        OCaml_fifo = {
             interpreter = "utop"
            }
        }
    }
})
```

### REPL-like behavior

```lua
require'sniprun'.setup({
    repl_enable = { "OCaml_fifo" }
})
```
