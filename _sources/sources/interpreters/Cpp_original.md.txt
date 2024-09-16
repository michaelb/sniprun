## Cpp original

Limitations

- Will only look and load external imports (include) that are SYSTEM import;
  as sniprun does not have any way to differentiate between #include "math.h"
  (the system library) and #include "math2.h" (your custom header),
  it will NOT look for #include "....", but only #include \<....>
  (those are restricted to system libraries).

- Need `g++` (or specify another compiler in configuration)

```lua
require'sniprun'.setup({
    interpreter_options = {
        Cpp_original = {
            compiler = "clang --debug"
            }
        }
    }
})
```
