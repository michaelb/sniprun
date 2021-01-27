# Python3_jupyter

Due to the requirement of `jupyter-kernel` and `jupyter-console`, REPL behavior is disabled by default.
To enable it include this in your configuration file

`let g:SnipRun_repl_behavior_enable = ["Python3_jupyter"]`

As there is a different interpreter for Python, you may want to force the selection of Python3_jupyter with

`let g:SnipRun_select_interpreters = ["Python3_jupyter"]`


## Limitations

The code runs on a separate jupyter python3 kernel which will NOT interefere with your own running kernels.

However, mind that the usual limitations of such kernels still apply: max duration of execution, etc... but you probably don't have to pay too much attention to this.


The jupyter kernel also has a substantial overhead when it comes to running code, in addition to (well-managed) imports, that means the Python3_jupyter interpreter may feel a bit slow compared to others






























