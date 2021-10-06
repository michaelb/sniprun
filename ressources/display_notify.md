# nvim-notify as display option

This plugin : https://github.com/rcarriga/nvim-notify

must be installed in order to use this configuration option
Sniprun will use the global configuration of the plugin

To use it, configure sniprun with:

```
lua << EOF
require'sniprun'.setup({
    display = {"NvimNotify"],
})
EOF
```
