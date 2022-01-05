## v1.1.2
- auto detection of entry point for many languages
- CFLAGS and other variables
- Python3\_fifo plots work

## v1.1.1
- Fix terminal display issues
- Configurable display options

## v1.1.0
- TerminalWithCode display option (courtesy of @control13)
- Fix default interpreter issue
- Python3\_fifo venv support + doc + fix indented bloc failure

## v1.0.6
- fix output with escape sequences

## v1.0.5
- fix issue with REPL interpreters staying active after nvim exit
- isolate backend REPL from different neovim instances

## v1.0.4
- fix python3 fifo and sage interpreters empty line in indented bloc bug

## v1.0.3
- configurable filetypes

## v1.0.2
- fix issue with API functions & callbacks
- fix double checkhealt crash issue

## v1.0.1
- fix issue when using sniprun with an empty config

## v1.0.0
- LCOV coverage
- Many (non inherently REPL) interpreters accept cli arguments when invoked from `:%SnipRun <args>`
- Deperecate vimscript configuration
- Update documentation, README and example

## v0.5.10-orgmodepatch
- orgmode fixes (indented blocks)

## v0.5.10 
- SageMath & orgmode support
- API
- nvim-notify display method

## v0.5.9
- TypeScript support
- Better README & example.rs
- Import level for Python3\_fifo


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
- Old 'vroom' framework deprecated


## v0.4.9
- inline_message functionnality


## v0.4.8
- More complete test pipeline
- C++ & C upgrade to support level Import
- Social preview and various assets
- checkhealth


## v0.4.7
- the first useful vroom tests
- Fix the bug where the channel would broke and the terminate function had a typo
- Compilation error messages (nicely truncated) for C and Rust


## v0.4.6
- Remove 'cc' dependency which caused issues with precompiled GLIBC
- a lot of unit tests, and complete CI pipeline
- vroom framework for integration test ?


## v0.4.5
- Plug mappings + vim-repeat support
- Trailing characters fix
- SnipInfo centralize all the information, better help


## v0.4.4
- Jupyter kernel available in Python3_jupyter !
- fixes a bug in Python3_original that imported too much modules
- improved README
- first tests in CI pipeline


## v0.4.3
- Rust interpreter and example now have tests
- Julia intepreter
- example.rs interpreter


## v0.4.1
- VimL config options
- REPL for R and Bash


## v0.4.2
- Better release download system
- More complete CONTRIBUTING.md & revamped README
- Github Action & build passing badge


## v0.4.0
- Go, C++, Ruby, Haskell interpreter
- REPL-like behavior for the first time (Python)


## v0.3.1
- First tag release / MVP
- Lua, C, Python, Rust, Bash, Java, Javascript and many more languages are supported
- Lua_nvim interpreter, courtesy of @vigoux, the first external contributor
- Comprehensive doc and readme
- First interpreter (Python) gets Import support level
- Generic interpreter



## Initial commit
- Sniprun development started on 17 Aug 2020
