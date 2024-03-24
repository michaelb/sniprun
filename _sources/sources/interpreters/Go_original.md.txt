## Go original
the executable (go , llgo or whatever) executable used to _build_ the snippet can be configured via


```
require'sniprun'.setup({
    interpreter_options = {
        Go_original = {
            compiler = "gccgo"
            }
        }
    }
})
```


