# Lua API to sniprun [WIP]

You can use sniprun API from:

```lua

local sa = require('sniprun.api')

```

then, some functions are accessible, such as

```
sa.run_range(r_start, r_end, <filetype>, <config>)
sa.run_string(codestring, <filetype>, <config>)

```

(ranges are integers matching the (inclusive) line numbers, codestring a string, filetype (optionnal) must be a string such as 'python', config allows to override the default/user config)


You can register listeners that will be called upon (async) sniprun output:


```
sa.register_listener(custom_function)
```

where custom function is a function that take one unique argument: a table which contains at least two entries:

 - 'status' (a string that's either 'ok' or 'error' for now, but your function should accept & manage other values)
 - 'message' (also a string, maybe be mutliline)

(Simply put, registered functions are callbacks)



​
​

Thus, an example of such a function (imitating the 'Classic' display with 'uwu' tendencies) would be

```
local api_listener = function (d)
    if d.status == 'ok' then
	print("Nice uwu: ", d.message)
    elseif d.status == 'error' then
	print("Oh nyow! Somethuwuing went wyong: ", d.message)
    else 
	print("Whut is this myeow? I don't knyow this status type nyah")
    end
end

sa.register_listener(api_listener)
```


## Warnings

Beware, sniprun is still thightly coupled to the current nvim buffer & instance, but there should not be dependencies for non-REPL, and interpreters running under Bloc-Level. 

REPL-capable and Import level (or more) interpreter may fetch information from the current buffer
    


