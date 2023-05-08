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
You can change the width of the terminal by using the display option in the configuration:
```
  display_options = {
    terminal_scrollback = vim.o.scrollback, -- change terminal display scrollback lines
    terminal_line_number = false, -- whether show line number in terminal window
    terminal_signcolumn = false, -- whether show signcolumn in terminal window
    terminal_width = 45,       -- change the terminal display option width
  },
```


![](visual_assets/terminal.png)


If you also want to print the code being executed to the 'terminal', then use `"TerminalWithCode"` instead in the 'display' key.



![](visual_assets/terminalWithCode.png)

