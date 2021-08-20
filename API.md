# Lua API to sniprun [WIP]

You can use sniprun API from:

```lua

local sa = require('sniprun.api')

```

then, some functions [WIP] are accessible, such as

```
sa.run_range(r_start, r_end, <filetype>)
sa.run_string(codestring, <filetype>)

```

(ranges are integers matching the (inclusive) line numbers, codestring a string, filetype (optionnal) must be a string such as 'python')
