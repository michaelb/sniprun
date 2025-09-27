## Sage fifo 

This is a pipe-based implementation: you have to run sniprun once before
being able to send code snippets (configure an autcmd?)

A sage REPL is launcher in the background and won't quit
until you exit neovim (or after 10h).

This interpreter only works in REPL-mode (and behaves REPL-like by default)

two configurations keys are available:

```lua
require'sniprun'.setup({
    interpreter_options = {
        Sage_fifo = {
            interpreter = "sage",
            sage_user_config = 1, -- the actual value has no effect, only the presence of the key is important
            }
        }
    }
});
```

Sage\_fifo currently support auto-importing python imports
