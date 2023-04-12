## Python3 original

To get the REPL behaviour (inactive by default) working, you need to isntall the klepto python package: `pip install --user klepto`

Then, to enable the REPL behavior for python in your config file

```
require'sniprun'.setup({
    repl_enable = {'Python3_original'}
})
```

HOWEVER, if you're interested in a very stable and solid REPL python interpreter, to process bigger amount of data or import some exotic modules (not supported by klepto), get a look at the Python3_fifo interpreter.


With the REPL enabled, sniprunning a \* (star) import `from module import *` may not work, indeed the imports needs to be correctly saved/loaded by klepto. klepto manages variables, functions and modules but very special things may fail.

Without REPL enabled, your python snip's will be executed faster (and not increasingly slower) and the correctness/cleanliness of the inner working is garanteed. By setting this, you can be sure your snip's will run free of side-effects and anything you would not want.

With or without REPL, the star imports may also not be automatically fetched, even though normal imports will be. Python3_original has the 'Import' support level but that won"t work with star import, and I don't think we'll be able to make a workaround due to the philosophy 'run only what's necessary' of sniprun.



To use a custom python interpreter ( maybe python2, or a particular version?) you can provide the following interpreter options:, using a venv is also supported (provide one or several relative paths "../venv" etc.. may be necessary if you didn't open neovim at the root of the project)


```
require'sniprun'.setup({
    interpreter_options = {
        Python3_original = {
            interpreter = "python3.9",
            venv = {"venv_project1", "venv_project2", "../venv_project2"},
            }
        }
    }
})
```

