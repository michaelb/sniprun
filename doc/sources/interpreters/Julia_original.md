## Julia original

Simply needs the `julia` executable

REPL mode can be activated with this configuration:

```lua
require'sniprun'.setup({
    repl_enable = {'Julia_original'},
})
```

Julia_original supports several interpreter options, such as the interpreter
executable (absolute path or command in PATH, by default "julia"), and the
'project' (will be passed as --project=... to the executed command)

```lua
require('sniprun').setup({
    interpreter_options = {
        Julia_original = {
            project="." --# either a fixed absolute path, or "." for nvim's current directory (from echo getcwd()  )
                        --# This directory has to contain a {Project,Manifest}.toml !
            interpreter="/path/to/custom/julia"
        }
    }
}) 
```
