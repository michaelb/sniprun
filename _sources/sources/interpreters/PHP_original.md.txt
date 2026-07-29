## PHP original

PHP interpreter with pipe-based REPL-like functionality.

In REPL-mode, a PHP REPL is launched in the background and won't quit till you exit neovim.

```lua
require'sniprun'.setup({
    selected_interpreters = { 'PHP_original' },
    repl_enable = {'PHP_original'},
})
```

Please note that if your code produces output, but also an error or a warning, only the error/warning will be shown.

You can customize the PHP interpreter used:

```lua
require'sniprun'.setup({
    interpreter_options = {
        PHP_original = {
            interpreter = "php",
        }
    }
})
```
