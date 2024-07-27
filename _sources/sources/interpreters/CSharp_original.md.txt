## C# original

This interpreter require the `mono` toolbox to be installed:
`csc` and `mono` must be in your $PATH

a custom compiler can be specified :

```lua
require'sniprun'.setup({
    interpreter_options = {
        CSharp_original = {
             compiler = "csc"
             }
        }
    }
})
```
