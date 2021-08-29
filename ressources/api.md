The results are displayed via the [nvim-notify](https://github.com/rcarriga/nvim-notify) plugin


The color of the notification and the title reflect the status (ok, error)

```
lua << EOF
require'sniprun'.setup({
  display = { "NvimNotify" },
})
```
![](visual_assets/api.png)
