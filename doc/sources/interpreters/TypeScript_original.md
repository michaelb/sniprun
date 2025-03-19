## TypeScript original

Prior to NodeJS v22.6.0:
Install `ts-node`, usually installed from npm

`sudo npm install -g ts-node typescript`

---

If you have NodeJS v22.6.0+:

```lua
require'sniprun'.setup({
    interpreter_options = {
        TypeScript_original = {
            compiler = 'node'
            }
        }
    }
})
```

[^1]: `ts-node` and `typescript` packages are no longer needed for newer NodeJS versions
