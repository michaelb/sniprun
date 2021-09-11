A vertical split is opened to the right, and it display (non-interactively) sniprun output

Can be closed with `:SnipClose` (or a shortcut to `<Plug>SnipClose`)

Highlighting is not supported yet

if you experience wrapping of the header line '---- OK ---' due to the presence of a number column, you can (and should anyway) set 

```vim
autocmd TermOpen * setlocal nonu
```

in your configuration.



To activate, add "Terminal" to the 'display' key in sniprun configuration:

```
lua << EOF
require'sniprun'.setup({
  display = { "Terminal" },
})
EOF
```



![](visual_assets/terminal.png)


