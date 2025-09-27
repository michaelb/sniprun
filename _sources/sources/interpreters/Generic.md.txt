## Generic

This interpreter allows you to support virtually any language,
provided it's not too strange, at up to bloc-level

If you're trying to override an already-supported language,
add Generic to the list of selected interpreters:


```lua
require'sniprun'.setup({
    selected_interpreters = { 'Generic' },
})
```
to add support for, let's say, python2

```lua
require'sniprun'.setup({
    interpreter_options = {
        Generic = {
            error_truncate="long",                 -- strongly recommended to figure out what's going on
            MyPython2Config = {                    -- any key name is ok
                supported_filetypes = {"python2"}, -- mandatory
                extension = ".py",                 -- recommended, but not mandatory. Sniprun use this to create temporary files

                interpreter = "python2",           -- interpreter or compiler (+ options if any)
                compiler = "",                     -- exactly one of those MUST be non-empty
                }
            }
        }
    }
})
```

to also add support for, let's suppose it wasn't officially supported, C

```lua
require'sniprun'.setup({
    interpreter_options = {
        Generic = {
            error_truncate="long",
            MyPython2Config = {
                supported_filetypes = {"python2"}, 
                extension = ".py",                

                interpreter = "python2",         
                compiler = "",                  

                exe_name = "",
                boilerplate_pre = ""
                boilerplate_post = ""
                },

            my_super_c_config = {                
                supported_filetypes = {"c", "mysupercfiletype}, 
                extension = ".c",                

                interpreter = "",         
                compiler = "gcc -o my_main -O3",                     -- compiler (+ options if necessary) (current working directory is sniprun's work directory - next to sniprun's log in $XDG_CACHE_DIR)

                exe_name = "my_main",                                -- executable name, by default a.out (always in sniprun's work directory)
                boilerplate_pre = "#include <stdio.h>\nint main () {"  -- include this before code snippets
                boilerplate_post = "}"                                 -- include this after code snippets
                }
            }
        },

    selected_interpreters = {"Generic"}
    -- other sniprun options ...
})
```

### How the generic interpreter works

#### For interpreted languages ("interpreter" is set)

1. Sniprun receive a snippet of code to run
2. The snippet gets surrounded by boilerplate_pre and boilerplate_post
3. The whole thing is written to a file with the given extension, named `<exe_name>_src.<extension>`
4. Sniprun runs `<interpreter> <file>.<extension>` and displays the stdout/stderr

#### For compiled languages ("compiler" is set)

1. Sniprun receive a snippet of code to run
2. The snippet gets surrounded by boilerplate_pre and boilerplate_post
3. The whole thing is written to a temporary file with the given extension,
   named `<exe_name>_src.<extension>`
4. Sniprun runs `<compiler>  <exe_name>_src.<extension>` , and if this has a
   non-success status, displays the stderr
5. Sniprun runs `./<exe_name>` and displays the stdout/stderr

### Community examples for non-officially supported languages

(contribute on github to the file: doc/sources/interpreters/Generic.md)
