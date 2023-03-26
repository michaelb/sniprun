# Welcome to the contributing page

Just in case: to compile `cargo build --release`, to create and show the documentation, `cargo doc --open` ( open target/doc/sniprun/index.html from your browser if not automatic).

## Add support for a new language language

First, you should try out to configure the [Generic interpreter](https://michaelb.github.io/sniprun/sources/Generic.html#Generic.html#community-examples-for-non-officially-supported_languages). Someone may have figured it out already!

If you succeed in doing so yourself, please contribute to that page so other may benefit from it!

Otherwise, carry on:

### How hard it is?

Lemon squeezy easy. A developper midly familiar with Rust and the language to add support for can write a working bloc-support interpreter in 30min ( 13min is my best time, for C\_original to 1h30. You can then submit your proposed changes as a PR to the master branch.

Higher support levels gets exponentially harder (depends on the languages though), so you should start out with Bloc.

### Understanding the framework

What do I write, and where?

-> You only have to write a file in src/interpreters/ that has the name of the interpreter, by convention; \<language\_name\>\_\<differentiator\>.rs

---

Yeah cool but what _code_ goes inside?

-> Inside, you must define a struct that implement the **Interpreter** trait. Have a look at existing implementations to get the idea, though some current interpreters use pretty advanced features of Sniprun that you may not want to deal with. Prefer looking at 'Bloc' level support interpreters for a smooth start. Make sure to respect the [conventions](#conventions). The "example.rs" interpreter is a good starting point, with more comments to help you understand what's happening.

---
I just finished some changes, how do I test my code quickly?

-> compile `cargo build --release` and run `nvim --cmd "set rtp+=. -u NONE <testfile>` from the sniprun project root. You may want to remove the 'release' sniprun with your plugin manager in case your runtimepath (rtp) still loads up the release version instead of your development version.

---

Is _my_ code running?

-> Assert that the file type detected by Neovim is contained in your list of supported file types. If there is already a implementation for your filetype/language, set (temporarly) your max support level to "Selected", or run something like `:lua require'sniprun'.setup({selected_interpreters = {'<name>'}})` before `:SnipRun` . `SnipInfo` will then tell you what interpreter will be used on an opened file.

---

I need to debug, how ?

-> Use the `info!("here")` macro instead of `println!("here")`. This writes to the log file you can find in ~/.cache/sniprun/sniprun.log (or ~/Library/Caches/sniprun/sniprun.log on Mac). Beware, if you panic, the logger will stop writing.

---

Can I panic!() ?

-> Yes but preferably only when you encounter a fatal error (eg: you have to write a file, but there is no space left on the device).
Failing user code compilation or incorrect user code panicking should be handled via the SniprunError enum.

---

My interpreter does not produce any output..?!

-> It's because your code is panicking. (unwrapping a `None` or these kind of things). Check the logs at ~/.cache/sniprun/sniprun.log for any interrpution, and see where your code can panic! .

---

I need to import some external dependencies.

-> Add what you need to the src/interpreters/import.rs file, and the Cargo.toml if necessary.

---

I need more than one file to write complicated code...

-> You can have a subfolder alongside your file (same name to prevent confusion and conflicts) and put some other code inside as you see fit. See the example.rs file: inside your work\_dir, you are free to do whatever you want

---

Do I need to manage async running and compiling?

-> No, Sniprun takes care of that for you. You can implement a single-threaded synchronous code just like the D\_original interpreter.

---

My interpreter has some quirks that the end user should know

-> Document limitations and features in doc/interpreter\_name.md .

---

I lack the ReplLikeInterpreter trait implementation and don't want to do REPL-like behavior:

-> You don't have to do it but the boilerplate `impl ReplLikeInterpreter for MyInterpreter {}` is required. You can overwrite the default implementation later if you wish to do some REPL-like functionality.


---
My tests are inconsistent ..?!?

-> Rust tests are run in parallel, and therefore a race condition may occur when writing to files and compiling.
Run with `cargo test -- --test-threads=1` or use the #[serial] attribute which you will probably need to pass the CI pipeline anyway.

---
My tests fail in the CI pipeline

-> The CI has limited capabilities, especially about the REPL functionnality. Tag your non-working-in-CI tests with '#[cfg\_attr(feature = "ignore\_in\_ci", ignore)]'

---
I think I've done a good job, but am I ready to submit a PR?

-> You should check beforehand that the output of `cargo test --release` and your own tests are satisfying. You've added the proper and necessary tests, and have documented any edge case in doc/.

--- 
REPL - based ?
Python3\_fifo has a pipe (UNIX fifo) - based ReplLikeInterpreter implementation, that may be useful if your language has an interpreter with proper stdio support. See [CONTRIBUTING\_REPL.md](ressources/CONTRIBUTING_REPL.md) for more info.

### What's the deal with...

- Support Levels? Those exists to document what the interpreter supports to the end user. They are also used for higher (file, project and system) levels as if an interpreter detects it does not need a support level that high, it can set down its own level and hopefully be faster [ since it won't need to open all files etc...]. **You don't have to worry about this too much if you are just getting started**.

- Errors? When possible and sensible, functions like fetch(), build() and execute() should return either an Ok(\_) variant or a Err(SniprunError). Choose the error that most closely describe whatever migth cause your function to fail, and populate it with a String message if relevant.

* The imposed names? To simplify contribution (you only have to write a interpreter), the main program fetch new files and run functions of your interpreter. This is only easily possible if you types names match your file name, as I can get those easily but i would have to read them, guess what struct is the correct one should you have many....no, I rather do the `use file_name::file_name;` trick that just works. Also helps future contributors/users.

### Conventions

A program (struct with methods) that can fetch code, execute it and return the result is called an interpreter.

- The interpreter main file is named \<Language\_name\>\_\<Differentiator\>.rs; for example "Python3\_Lsp.rs", case-independent.
- The interpreter main file contains a struct has the **exact same name** as the file (minus the .rs extension).
- Names for interpreters should be unique. Include filenames, and also the name returned by `get_name()` that should be identical (case difference is tolerated).
- Extra files for the same interpreter go into a subdfolder alongside the interpreter's main file. The subfolder has the same name as the file, minus the extension.
- The interpreter try to follow (and create by itself) SupportLevel hints when possible; for example, will not try to parse an entire project into when it has been determined SupportLevel::Line is enough to run the submitted code. Don't worry if you don't get this yet for your first bloc-level contribution.
- The interpreter should try not to panic, it'll be nicer if the various errors can be converted and returned as SniprunError as defined in src/error.rs and suggested by the Interpreter trait

## Contribute to Sniprun itself

Well you are welcome to submit a PR, as long as you mind those points:

- Your changes do not break any interpreter, even partially.
- If needed (eg for when your changes touches a core part of Sniprun such as the DataHolder), you have tested your changes with every interpreter (or the CI pipeline did).

## Sniprun Mindset

To pay attention to, when writing an interpreter or changes:

- **Minimum code retrieval** : Sniprun should only fetch from the buffer/file the bare minimum necessary to get working.
- **Allow snips from incomplete files** : if you need to read a bigger part of the file than the data provided by sniprun (in DataHolder), you should NOT fail because the file miss a '}' 35 lines after the code snip.
- **IO optimization** : it's OK if you write 3 files each time sniprun fires. It's not OK if you re-index a whole project and write a 50Mo file. Overall this is a pretty relaxed rule, as most code sent to sniprun (to then write etc...) is very short, a few lines at most.
- **Code clarity** : at least comments for non-trivial parts, 'good code' is given value even if I get, and did that myself, than sometimes dirty hacks are necessary.
- **Documentation** : not extensively required, but limitations and subtilities, if any, of your interpreter should be written a the doc/interpreter_name.md file: that will be accessible through :SnipInfo [name] this way!
