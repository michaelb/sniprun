This interpreter is currently a work in progress and is probably not usable

The Prolog interpreter supports setting a different compiler/interpreter for prolog such as swi ('swipl')

you can set it with the following key:


```
require'sniprun'.setup({
    interpreter_options = {
        Prolog_gnu = { interpreter = "swipl" }
        }
    }
})
```
