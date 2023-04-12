## F# fifo

### This interpreter relies on dotnet fsi being available and on your path


The default interpreter command is `dotnet fsi --nologo` but it can be changed via the configuration key


```
require'sniprun'.setup({
    interpreter_options = {
        FSharp_fifo = {
             interpreter = "...."
            }
        }
    }
})
```


### REPL (would solve slowness issues)

For now, REPL is broken due to dotnet fsi being capricious about its stdin.

I'll explain rapidly how sniprun implement a REPL interpreter around named pipes (FIFOs).

The first time a fifo-based interpreter receive a run command, it forks to the background and executes `ressources/init_repl.sh`.
There is a lot of thing in that script but to replicate, you just have to:



- `mkfifo pipe_in`

- create a launcher script:

```bash
#!/bin/bash
/bin/cat pipe_in | dotnet fsi 

# or replace 'dotnet fsi' by whatever you cant to try
```

- launch it in the background: `bash ./launcher.sh &`, (or `bash ./launcher.sh > out.txt & ` to redirect stdout to out.txt like sniprun does)

- ensure the pipe will stay open: `sleep 3600 > pipe_in &` (cat, exec 3> variations will also work)

- `echo "printfn \" hey \" " > pipe_in` or `cat hello_world.fsx > pipe_in`

- normally, the result should be printed in the terminal that ran the launcher, or in the out file.




#### The issue:

right now, dotnet fsi looks like it's blocked by the first sleep > pipe_in... but something **has** to keep the pipe open or when it closes, the fsi REPL reading from that will exit.

I suspect the thing has something to do with interactive mode. 

For example, `python` has a similar problem, but `python -i ` (forced interactive mode, even if no terminal is detected because it runs in the background / its stdin was hijacked) works fine in the above example.

If you find something to replace dotnet fsi with, that exhibits the same correct behavior as `python -i`, sniprun REPL mode _should_ work.

