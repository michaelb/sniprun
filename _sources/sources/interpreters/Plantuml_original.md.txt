## PlantUML original

This interpreter relies on `plantuml`, which needs to be available on the $PATH

This interpreter supports the following options:

```lua
require'sniprun'.setup({
    interpreter_options = {
        Plantuml_original = {
             output_mode = "-tutxt", --# one of the options allowed by plantuml
             compiler = "/home/user/my_custom_plantuml_install/plantuml"
            }
        }
    }
})
```

You can add options to the 'compiler' key, but do not
set "-pipe" (or it'll break output), and "-failfast2",
"-nbthread auto" are already set.
