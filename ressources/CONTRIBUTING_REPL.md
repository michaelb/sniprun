# Making a REPL-capable interpreter for sniprun

## Is it possible ?

Yes, most of the time, if the language already has an available interpreter. It _could_ be possible otherwise but has yet to be really done.

To avoid confusion, we'll call the language interpreter 'interpreter', and sniprun's part (implementing the Interpreter trait) the runner.

## How ?
 Two ways, mostly. Either:
 - your language has 'quirks' (like for R and Python with the klepto module, see R\_original and Python3\_original) that allow current variables and stuff to be 'saved' to a file then loaded

 or

 - you make use of a named pipe (fifo) and pipe what sniprun says into it. the pipe is connected to a live, running, interpreter for your language. Its output is written to a file and sniprun waits for landmarks (start, end) to be printed. 


 I strongly advise the latter methodology, which has several advantages that I won't discuss here, but can be harder to implement if your language's interpreter has weird stdin/stdout/stderr behavior. Like non-disablable prompts printed to stdout.


## How to implement a pipe-based repl-capable runner

The best example I'm going to discuss is Python3\_fifo, even if it's a bit bloated from python-specific things.

Just like you implemented the Interpreter trait for a conventional runner, you'll have to implement the ReplLikeInterpreter trait. Another trait (InterpreterUtils) is automatically implemented and provides features & data persistency to help you survive across different/independent runs. 

1. Running something in the background:
    
    Unfortunately, this require a first workaround. It's mainly due to how sniprun can't really launch a background process that stays alive, even when the thread executing the user's command exits, and sorta re-launch itself some time later (the interpreters needs some time to launch) to execute the input. The first user command will always fail with a message ("launching .. interpreter in the background, please re-run last snippet").
    ```rust
      fn fetch_code_repl(&mut self) -> Result<(), SniprunError> {
        if !self.read_previous_code().is_empty() {
            // nothing to do, kernel already running

	    ....

            self.fetch_code()?;
            Ok(())
        } else {

	    let init_repl_cmd = self.data.sniprun_root_dir.clone() + "/ressources/init_repl.sh";
          
            match daemon() {
                Ok(Fork::Child) => { // background child, launch interpreter
                    let _res = Command::new("....."); // bash init_repl_cmd args

		    let pause = std::time::Duration::from_millis(36_000_000);
                    std::thread::sleep(pause);

                    return Err(SniprunError::CustomError("Timeout expired for python3 REPL".to_owned()));
                }
                Ok(Fork::Parent(_)) => {} // do nothing
                Err(_) => info!(
                    "Python3_fifo could not fork itself to the background to launch the kernel"
                ),
            };

            let pause = std::time::Duration::from_millis(100);
            std::thread::sleep(pause);
            self.save_code("kernel_launched\nimport sys".to_owned());

            Err(SniprunError::CustomError(
                "Python3 kernel launched, re-run your snippet".to_owned(),
            ))
        }
    ```
    The important thing to note is that `self.read_previous_code()` is used to determine whether a kernel was already launched; (`self.get_pid()/set_pid()` can be used to store an incrementing number of 'runs' or the child's PID, or whatever.

2. Landmarks

```rust
fn add_boilerplate_repl(&mut self) -> Result<(), SniprunError> {
        self.add_boilerplate()?;
        let start_mark = String::from("\nprint(\"sniprun_started_id=")
            + &self.current_output_id.to_string()
            + "\")\n";
        let end_mark = String::from("\nprint(\"sniprun_finished_id=")
            + &self.current_output_id.to_string()
            + "\")\n";
        let start_mark_err = String::from("\nprint(\"sniprun_started_id=")
            + &self.current_output_id.to_string()
            + "\", file=sys.stderr)\n";
        let end_mark_err = String::from("\nprint(\"sniprun_finished_id=")
            + &self.current_output_id.to_string()
            + "\", file=sys.stderr)\n";
	....
```

the user's code has to be wrapped with 4 landmarks that prints 'start run°X', 'end run n°X' messages. Snipruns uses them to determine when the user's code has finished executing. It's then displayed. Note that things can't be displayed 'live', and if someone launches an infinite loop, they won't have any output.


3. Waiting for output
``` rust
fn wait_out_file (....){
    loop {
	std::thread::sleep( 50 ms);

	//check for content matching the current ID in file for stderr

	//check for content matching the current ID in file for stdout
	
	//break when something found & finished
    }
}
```
is executed & returned at the end of `execute_repl` that firsts send the user's snippet (wrapped with landmarks) to the FIFO pipe.

4. Helper scripts
Though not very documented, the `ressources/init_repl.sh` and `ressources/launcher.sh` script are resuable for other runners than Python3\_fifo (see Mathematica that has its own similar scripts in `src/interpreters/Mathematica_original/`. They take care of plugging together the fifo, stdout, stderr files and the interpreter's process. They also take care of closing the interpreter (and free the ressources) when nvim exits


### End notes:
- you should take care of separating the working directories for repl-capable interpreters from different neovim sessions, to avoid having nonsense because of mixed fifo and output files content:

```
fn new_with_level(...)
 Box::new(Python3_fifo {
            cache_dir: data.work_dir.clone() + "/python3_fifo/" + &Python3_fifo::get_nvim_pid(&data),
	    ....
```

- disable prompts for your interpreter. They'll pollute stdout. For example, in python, you'll have to set `sys.ps1` and `sys.ps2` to `""`.

