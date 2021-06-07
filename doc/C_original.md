gcc is recommended, for that it's able to detect, compile and run nested functions, however you can change the default compiler with:


```
require'sniprun'.setup({
    interpreter_options = {
        C_original = {
             compiler = "clang"
            }
        }
    }
})
```

This interpeter will also only import (all)  #include \<...> but not any #include "..."
