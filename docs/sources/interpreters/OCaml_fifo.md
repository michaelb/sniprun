## OCaml fifo

This interpreter relies on `ocaml` by default, but has been confirmed to work with  `utop` in normal (non-REPL) mode.



The default interpreter can be changed relatively safely for normal (non-REPL) mode:
```
require'sniprun'.setup({
    interpreter_options = {
        OCaml_fifo = {
             interpreter = "utop"
            }
        }
    }
})
```
You can change those value, (to use `clj` for example ?) but it could break sniprun anytime



### REPL-like behavior

```
require'sniprun'.setup({
    repl_enable = { "OCaml_fifo" }
})
```
