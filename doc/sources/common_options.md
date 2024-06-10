## Interpreter configuration

You can select an interpreter (with its name from `:SnipInfo`) if several of them support the language/filetype
(see {ref}`use_on_filetype <use-on-filetype>` if necessary) and you want a specific one

```
lua << EOF
require'sniprun'.setup({
    selected_interpreters = { "Some_interpreter", "Python3_fifo"},
})
EOF
```

Many interpreters have REPL capabilities that are _not_ enabled by default. You can turn this behavior on (or off) with the `repl_enable` and `repl_disable` keys, that similarly take interpreters names as arguments:


```
lua << EOF
require'sniprun'.setup({
    repl_enable = {"Python3_original", "Julia_jupyter"},   --# enable REPL-like behavior for the given interpreters
    repl_disable = {"Lua_nvim"},          --# disable REPL-like behavior for the given interpreters
})
EOF
```


## Common options

Every interpreter supports getting documentation via `:SnipInfo <interpreter_name>`

To specify interpreter options, you have to add the following to your sniprun config:



```
lua <<EOF
require('sniprun').setup({

    interpreter_options = {
        <Interpreter_name> = {
            some_specific_option = value,
            some_other_option = other_value,
        }
    }
  }, 
})
EOF
```

For example:

```
lua <<EOF
require('sniprun').setup({

    interpreter_options = {         --# interpreter-specific options, see doc / :SnipInfo <name>

        --# use the interpreter name as key
        GFM_original = {
            use_on_filetypes = {"markdown.pandoc"}    --# the 'use_on_filetypes' configuration key is
                                                      --# available for every interpreter
        },
        Python3_original = {
            error_truncate = "auto"         --# Truncate runtime errors 'long', 'short' or 'auto'
                                            --# the hint is available for every interpreter
                                            --# but may not be always respected
        }
  }, 
})
EOF
```

(use-on-filetype)=
### The "use_on_filetypes" key

The `use_on_filetypes` key is implicitely an option of every interpreter

```
interpreter_options = {
    GFM_original = {
        use_on_filetypes = {"markdown.pandoc", "rstudio" }
    }
}
```

### The "error_truncate" key

Also available for every interpreter if you don't like how sniprun truncate some outputs by default (auto), but it will not have an effect on all interpreters.

```
interpreter_options = {
    Python3_original = {
        error_truncate = "auto"     --# Truncate runtime errors 'long', 'short' or 'auto'
    }
}, 
```

## The interpreter/compiler keys

Almost every interpreter support either the "interpreter" or "compiler" key even if not explicitely documented, depending on whether they're about an interpreter or compiled language.

example:

```
interpreter_options = {
    Python3_original = {
        interpreter = "python3.9"
    }
    Rust_original = {
        compiler = "/home/user/bin/rustc-patched -Zlocation-detail=none"
    }
}, 
```

You can see what interpreters/compilers are being used at any time by watching sniprun's log for the line
"using compiler XXXX" or "using interpreter XXXX" when you run a snippet.
While options can (generally) be added to these interpreters/compilers strings, mind that some options are often already passed, and
sometimes mandatory (ex: "-o main_file_name", "--nologo") and whatever is added can mess up the format
sniprun internally expect, or be straight out incompatible with the formers. Be careful!

Exceptions:
 - Scala_original has both interpreter and compiler keys that should be set consistently with each other
 - *_jupyter, Generic, GFM_original, Orgmode_original, and Neorg_original do not support any of these keys, as they rely on the interpreter for the code's block language.


