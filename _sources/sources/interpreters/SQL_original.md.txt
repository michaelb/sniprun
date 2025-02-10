## SQL original

This interpreter relies on `usql` being installed

```lua
require'sniprun'.setup({
    interpreter_options = {
        SQL_original = {
             interpreter = "/home/user/my_usql_install/usql --myoption"
            }
        }
    }
})
```

the option "-w" (do not prompt for the password) and --file are already passed
by sniprun, so you should not pass conflicting/duplicate options.

This interpreter will prompt you at first use what database you want to connect to,
that is an address (including user & password if applicable) as you would
pass to `usql` itself. In case the database is local (such as for sqlite),
you should input an absolute path or a path relative to neovim's current
working directory (`:pwd`).

This address (and the possible user/password) is NOT stored anywhere, but sniprun
will remember it as long as the neovim session stays open. You can use `:SnipReset`
to clear sniprun's memory.
