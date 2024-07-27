## C original

`gcc` is recommended, for that it's able to detect, compile and run nested
functions, however you can change the default compiler with:

```lua
require'sniprun'.setup({
    interpreter_options = {
        C_original = {
             compiler = "clang"
            }
        }
    }
})
```

If you run with GCC, Sniprun will be able to run function + code in the
same snippet, or functions + main() function regardlessly, but only the
latter is supported by `clang`.

This interpreter will also only import (all)  #include \<...> but not
any #include "..." (systems-wide include only, not the headers from your
project, unless the environment variable `$C_INCLUDE_PATH` or
`$CPLUS_INCLUDE_PATH` have been set). In this case, please make sure those
variable cover **ALL** the paths needed to fetch every header file `#include`'d


the C\_original interpreter will also make use of the following environment variables:

- `$C_INCLUDE_PATH`
- `$C_PLUS_INCLUDE_PATH`
- `$LIBRARY_PATH`
- `$CFLAGS`


and will add them to the build options it uses.
Please specify _absolute paths_, and not relative ones!


Using a tool such as [direnv](https://direnv.net/) may be really useful
to set up those variables when you `cd` into your project' directory.
