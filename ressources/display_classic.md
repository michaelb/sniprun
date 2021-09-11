The result is displayed at the bottom of the window, in the command-line area.

Errors are displayed in red, and multiline output is supported



Activated by default, to change activate, add or remove "Classic" to the 'display' key in sniprun configuration:

```
lua << EOF
require'sniprun'.setup({
  display = { "Classic" },
})
EOF
```


![](visual_assets/classic.png)
