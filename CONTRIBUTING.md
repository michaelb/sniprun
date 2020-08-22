#Welcome to the contributing page

## Add support for your language

### How hard it is?

Lemon squeezy easy. A developper midly familiar with Rust and the language to add support for can write a working bloc-support interpreter in about 1h30.

Higher support levels gets exponentially harder (depends on the languages though).

### Understanding the framework

What do I write, and where?

-> You have to write a file in src/interpreters/ that has the name of the interpreter, by convention; \<language_name\>\_\<differentiator\>.rs

Yeah cool but what _code_ goes inside?
-> Inside, you must write a struct that has the **exact same name** as the file (minus the .rs extension).
The struct must implement the **Interpreter** trait. Have a look at existing inmplementations to get the full picture.

I just compiled, how do I test my code quick?
-> run `nvim -u plugin/sniprun.vim some_test_file.ext` from sniprun project root.

If _my_ code running?
-> Assert that the file type detected by vim is contained in your list of supported file types. If there is already a implementation for your filetype/language, set (temporarly) your max support level to "Selected".

I need to debug, how ?
-> Use the `info!("here")` macro instead of `println!("here")`. This writes to the log file you can find in ~/.cache/sniprun/sniprun.log.

Can I panic!() ?
-> Yes but preferably only when you encounter a fatal error (eg: you have to write a file, but there is no space left on the device).
Failing compilation or failing to run the code should be handled via the SniprunError enum

I need to import some exterior dependencies.
-> Add what you need to the src/interpreters/import.rs file, and the Cargo.toml if necessary

I need more than one file to write complicated code...
-> You can have a subfolder with the same name as your file (to prevent confusion and conflicts) and put some other code inside as you see fit.

## Contribute to Sniprun itself

Well you are welcome to submit a PR, as long as you mind those points:

- Your changes do not break any interpreter, even partially.
- If needed (eg for when your changes touches a core part of Sniprun such as the DataHolder), you have tested your changes with every interpreter.
