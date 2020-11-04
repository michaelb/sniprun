To get the REPL behaviour (active by default) working, you need to isntall the klepto python package: `pip install --user klepto`

Alternatively, disable the REPL behaviour for python in your config file

let g:SnipRun_repl_behavior_disable = ["Python3_original"]

With the REPL enabled, sniprunning a \* (star) import `from module import *`  may not work, indeed the imports needs to be correctly saved/loaded by klepto. klepto manages variables, functions and modules but very special things may fail.

With or without REPL, the star imports may also not be automatically fetched, even though normal imports will be. Python3_original has the 'Import' support level but that won"t work with star import, and I don't think we'll be able to make a workaround due to the philosophy 'run only what's necessary' of sniprun.
