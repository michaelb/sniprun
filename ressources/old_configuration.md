Sniprun is compatible with its old configuration keys, but they may be deprecated at some point.

## I created this for people who had configured sniprun in the old days and just want to quickly change it, without switching to lua config etc..


- select an interpreter over another: `let g:SnipRun_select_interpreters = ["interpreter_name", "another_one"]`
- enable REPL behavior for the given interpreter: `let g:SnipRun_repl_behavior_enable = ["interpreter_name"]`
- disable REPL behavior for the given interpreter: `let g:SnipRun_repl_behavior_disable = ["interpreter_name"]`
- change the display mode (echomsg/inline): `let g:SnipRun_inline_messages = 0     "default, or 1`
