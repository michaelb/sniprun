## Generic

This interpreter allows you to support virtually any language, provided it's not to strange, at up to bloc-level

If you're trying to override an already-supported language, add Generic to the list of selected interpreters:


```
require'sniprun'.setup({
    selected_interpreters = { 'Generic' },
})
```

to add support for, let's say, python2


```
require'sniprun'.setup({
    interpreter_options = {
        Generic = {
            MyPython2Config = {                    -- any key name is ok
                supported_filetypes = {"python2"}, -- mandatory
                extension = ".py",                 -- recommended, but not mandatory. Sniprun use this to create temporary files

                interpreter = "python2",           -- interpreter or compiler (+ options if any)
                compiler = "",                     -- one of those MUST be non-empty
                }
            }
        }
    }
})
```

to also add support for, let's suppose it wasn't officially supported, C


```
require'sniprun'.setup({
    interpreter_options = {
        Generic = {
            MyPython2Config = {
                supported_filetypes = {"python2"}, 
                extension = ".py",                

                interpreter = "python2",         
                compiler = "",                  

                exe_name = "",
                boilerplate = ""
                },

            my_super_c_config = {                
                supported_filetypes = {"c", "mysupercfiletype}, 
                extension = ".c",                

                interpreter = "",         
                compiler = "gcc -o my_main -O3",                     -- compiler (+ options if necessary) (current working directory is sniprun's work directory)

                exe_name = "my_main",                                -- executable name, by default a.out (always in sniprun's work directory)
                boilerplate_pre = "#include <stdio.h>\nint main () {"  -- include this before code snippets
                boilerplate_post = "}"                                 -- include this after code snippets
                }
            }
        },

    -- other sniprun options, for example:
    selected_interpreters = {"Generic"}
})
```



### Community examples for non-officially supported languages

(contribute here)
