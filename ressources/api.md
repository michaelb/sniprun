The results are displayed via the [nvim-notify](https://github.com/rcarriga/nvim-notify) plugin


The color of the notification and the title reflect the status (ok, error)

```
lua << EOF
require'sniprun'.setup({
  display = { "NvimNotify" },
})
```
![](visual_assets/api.png)


Changing the contents of the buffer will generally not interfere with sniprun with the exception of running multiple code blocs in a markup language (such as markdown or orgmode), because sniprun gets the list of the positions of the code blocs once, before running & displaying once per code bloc
