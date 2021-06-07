## v0.5.8
- Mathematica support
- FIFO - based REPL works ok (Mathematica, Python)!
- Better help/snipinfo formatting

## v0.5.7
- Julia jupyter interpreter is up (though a bit complicated to setup)
- Configurable windows borders

## v0.5.6
- Automated publish system by github actions
- Binary now compatible with pre-GLIBC 2.33 systems


## v0.5.5
- Configurable colors
- Use current buffer instead of save files for import level

## v0.5.4
- Support configuring compiler, interpreter or venv (python)
- Fine-tune display modes + highlight
- Better namespace management for C++ (courtesy of Dan Green)
- Ping binary to check compatibility
- Fix major bug with download system

## v0.5.3
- Display results in "Classic', "VirtualText", "Floating Windows", and "Terminal" mode
- Checkhealth pings binary to check compatibility
- Official Mac (incl. M1) support
- Fix for the AUR packaged version


## v0.5.2
- Ada and Scala interpreters
- Operator mode, courtesy of @DarwinSenior
- Julia REPL via jupyter kernel
- Compatibility fix for nvim 0.4.x (will miss out on features >= sniprun v0.4.9)


## v0.5.1 
- Markdown interpreter
- Better :SnipInfo
- SnipInfo \<name> now display the help file in doc/

## v0.5.0
- Sniprun becomes a Lua (+Rust) plugin!!
- Lua backend, config, and checkhealth
- Fully backward compatible with old configs

## v0.4.9
- inline_message functionnality

## v0.4.8
- More complete test pipeline
- C++ & C upgrade to support level Import


## v0.4.7
- the first useful vroom tests
- Fix the bug where the channel would broke and the terminate function had a typo
- Compilation error messages (nicely truncated) for C and Rust

## v0.4.6
- Remove 'cc' dependency which caused issues with precompiled GLIBC i think
- a lot of unit tests


## v0.4.5
- Plug mappings + vim-repeat support
- Trailing characters fix


## v0.4.4
- Jupyter kernel available in Python3_jupyter !
- fixes a bug in Python3_original that imported too much modules
- improved README

## v0.4.3
- Rust interpreter and example now have tests


Sniprun development started on 17 Aug 2020
