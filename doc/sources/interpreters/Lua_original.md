## Lua original

Limitation/feature

IF
- your code selection contains "nvim' or "vim", even in comments,
- you haven't explicitely selected Lua_original
- your code snippet fails

THEN

Sniprun will use the lua-nvim interpreter instead of the normal 'lua' one.
