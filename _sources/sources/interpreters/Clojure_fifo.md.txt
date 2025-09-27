## Clojure fifo

This interpreter relies on `clojure`

The default interpreter command is `clojure`
(or `clojure -e "(clojure.main/repl :prompt (defn f[] ("")) )"`, that allow
not displaying the repl prompt) but it can be changed via the configuration key

 The defaults are equivalent to specifying:

```lua
require'sniprun'.setup({
    interpreter_options = {
        Clojure_fifo = {
             interpreter_repl = "clojure -e \"(clojure.main/repl :prompt (defn f[] (\"\")) )\""
             interpreter = "clojure"
            }
        }
    }
})
```

You can change those values, (to use `clj` for example ?)
but it could break sniprun anytime

### Blocked REPL

Clojure is a bit capricious and sometimes a typo will block forever
(and a timeout will appear after 30s). 
Don't hesitate to `SnipReset`, even though it will lose all
the currently in-memory variables...
