## Mathematica original

### QuickStart

Check that `WolframKernel` and `pkill` are installed and on your path.

Then, the setup 99% of people want:

```
lua require'sniprun'.setup({
    repl_enable = {'Mathematica_original'},
    interpreter_options = {
         Mathematica_original = {
            use_javagraphics_if_contains = {'Plot'}, -- a pattern that need <<JavaGraphics
        },
    },
})
```





### Enabling graphics

You can specify whether the ``<<JavaGraphics` `` command ought to be issued before running a snippet that contains some patterns, like 'Plot'. Enabling this may create a significant delay when the graphics are switched to that mode.


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

As a general rule of thumb, in non-REPL mode, either sniprun a Plot _or_ normal statements




### Print on each sniprun (non-REPL only!)

To make the experience more notebook/REPL -like, those options (incompatible with the true REPL mode) can be configured.

They will wrap any/the last line, if they dont contain alread a Print, Plot or end with ";" or and open bracket

!! WARNING !! This can lead to dangerous side-effects, mathematica contains very little documentation about this.
To feel safe, you wouldn't use these unless you only execute code line-by-line. It may or may not work with blocs.

```
lua require'sniprun'.setup({
    interpreter_options = {
        Mathematica_original = {
            wrap_all_lines_with_print = false,       -- wrap all lines making sense to print with Print[.];
            wrap_last_line_with_print = false,       -- wrap last line with Print[.]
        },
    },
})
```




### Quirks of REPL-mode


Enabling REPL for mathematica will launch a WolframKernel instance in the background. This session will close when the last neovim instance quits, but is only usable by one neovim instance.
I hope you only need to sniprun mathematica snippets in one neovim instance, because anything else will probably crash.

In REPL-mode, your first SnipRun command of the neovim instance is used to start the REPL kernel, and the selection is discarded. You'll have to re-run it, sorry, but that's the only way for now.

In REPL-mode, just like in the normal interpreter, suffix your expressions with ';' if you don't want them to output something. I strongly recommend to suffix Plots with ';'



### Troubleshooting

- No valid password found

  - For some reason, WolframKernel doesn't like being launched in the background if there is arleady a WolframKernel instance running. You can close your own, or `killall -9 WolframKernel` to kill all the WolframKernel still running, and re-open neovim. This will _also_ close the kernels you've launched yourself!



