To use the mathematica_original interpreter, you need to have `wolfram` (a shortcut to WolframKernel) installed and on your PATH.

# Enabling graphics

You can specify whether the "<<JavaGraphics` command ought to be issued before running a snippet that contains some patterns, like 'Plot'. Enabling this may create a significant delay when the graphics are switched to that mode.


```
lua require'sniprun'.setup({
    interpreter_options = {
	Mathematica_original = {
	    use_javagraphics_if_contains = {'Plot', 'SomeCustomPlt'}, -- a pattern that need <<JavaGraphics
	    keep_plot_open_for = 3600,    -- a positive integer -> how many seconds to keep the plot window open
	},
    },
})
```

This can be useful if you didn't already use the construct:

`Plot[Sin[x], {x, 0, 10}, DisplayFunction -> CreateDialog]`


If your selection contains a Plot (or matching pattern), in non-REPL mode Mathematica_original will never return (in order to keep the graph window open), this means other statements will not return output.

As a general rule of thumb, either sniprun a Plot _or_ normal statements
