Display the resutls in a temporary floating window.

The floating window is closed on the CursorMoved event, or from `:SnipClose`

The highlight groups used are :
- "SniprunFloatingWinOk"
- "SniprunFloatingWinErr"


You can configure the displau key as shown to enable temporary floating windows display mode

```
require'sniprun'.setup({
  display = { "TempFloatingWindow" },
})
```

OR  (enable only for long outputs)

```
require'sniprun'.setup({
  display = { "LongTempFloatingWindow" },
})
```
![](visual_assets/floating_window.png)

