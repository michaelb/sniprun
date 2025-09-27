## Python3 fifo

This is a pipe-based implementation that has some quirks:

You have to run sniprun once before being able to send code snippets
to it (configure an autocmd?)

A python REPL is launched in the background and won't quit till you exit neovim.

This interpreter only works in REPL-mode, and is not the default for Python
files, so to use it you should configure it as following:

```lua
require'sniprun'.setup({
    selected_interpreters = { 'Python3_fifo' },
    repl_enable = {'Python3_fifo'},
})
```

if a snippet produce an error important enough to crash the interpreter,
you may be required to re-launch the kernel (with a `SnipRun`)

setting a custom python interpreter and venv is also supported

```lua
require'sniprun'.setup({
    interpreter_options = {
        Python3_fifo = {
            interpreter = "python3.9",
            venv = {"venv_project1", "venv_project2", "../venv_project2"},
            }
        }
    }
})
```
