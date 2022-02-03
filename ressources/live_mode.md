# What is sniprun's live mode ?

The live mode hook the SnipRun command to the TextChanged event, meaning that at every change to make to the buffer, the current line will be sent to sniprun for evaluation. This can mean a lot of times, especially if you type fast.

The result is a virtual text, displaying at the end of the current line that print the result (stdout) of the line. Nothing is displayed when the line is incomplete / incorrect, a bit like codi.

# Warnings

The live mode **will execute code you didn't think really about** (and by that I mean even less than usual)
Thus:
 - Your code will get executed **lots** of times; check that your CPU can keep up. Even a slow 60wpm typing can make a Rust program recompile 3x per second, which is also different from sending 3 string/s to a running REPL.
 - Sniprun will try to execute even incomplete lines. You hadn't finished typing that `rm /path/to/arghhh` ? sniprun' not aware and removed the parent directory. Whoops. For these reasons, I strongly suggest to:
    - never run bash/shell with live mode
    - disable the live mode whenever your code modifies files or invoke system commands.

If you're running a REPL-capable interpreter, while it'll probably work, mind that:
- the REPL will have to gulp a lot of incomplete code without crashing and stuff
- typing b = b  + 1 + 1 will increment b by more than 2 !! (since an intermediate b=b+1 is valid and thus changes b before b=b+1+1)

# Enable and usage

`live_mode_toggle='enable'` in the config, (set to either 'enable' or 'off' - the default -, the disreptancy is only to force people to come here and read the warnings in case some smart kid want to skip and just set it to 'on') 

and then use the :SnipLive command and start coding.

