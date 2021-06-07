This is a pipe-based implementation that has some quirks:


You have to run sniprun once before being able to send code snippets to it (configure an autocmd?)

A python REPL is launched in the background and won't quit till you exit neovim.


This interpreter only works in REPL-mode, and is not the default for Python files, so to use it you should configure it as following:


```
require'sniprun'.setup({
    selected_interpreters = { 'Python3_fifo' },
    repl_enable = {'Python3_fifo'},
})
```
