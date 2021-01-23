# Sniprun

![](https://img.shields.io/badge/sniprun-v0.4.2-green.svg) ![](https://github.com/michaelb/sniprun/workflows/Rust/badge.svg)

Sniprun is a code runner plugin for neovim. It aims to provide stupidly fast partial code testing for interpreted **and compiled** [languages](#support-levels-and-languages) . Sniprun blurs the line between standard save/run workflow, jupyter-like notebook, unit testing and REPL/interpreters.

- [Demos](README.md#demos)
- [What does it do ?](README.md#what-does-it-do-)
- [A quick word on REPL-like behavior](README.md#a-quick-word-on-repl-like-behavior)
- [Installation](README.md#installation)
  - [Prerequisites &amp;&amp; dependencies](README.md#prerequisites--dependencies)
  - [Install Sniprun](README.md#install-sniprun)
- [Usage](README.md#usage)
  - [Running](README.md#running)
  - [Stopping](README.md#stopping)
  - [REPL-like behavior](README.md#repl-like-behavior)
  - [Configuration](README.md#configuration)
  - [My usage recommandation &amp; tricks](README.md#my-usage-recommandation--tricks)
- [Support levels and languages](README.md#support-levels-and-languages)
- [Known limitations](README.md#known-limitations)
- [Contribute](README.md#contribute)
- [Related projects](README.md#related-projects)

## Demos

![](demo.gif)

(the exact same thing can also be done on **compiled** languages such as Rust, to the relevant support level's extent). A few lines of code (which maximum semantic complexity depends on the support level) are now within a print statement's reach.

![](demo_rust.gif)

send-to-REPL-like behavior is available for Python, R (both real REPLs) and Bash (simulated), coming soon for many other interpreted and compiled languages.

![](demo_repl.gif)

> Note: SnipRun is still under development, so expect new features to be introduced quickly, but also some other things may change and break your workflow.

> **Note: Python users are required to install the 'klepto' python package or to disable REPL behavior** in their config files (to get the same behaviour than before 0.4.1)

## What does it do ?

Basically, it allows you to run a part of your code, even if as-is your code won't even compile/run because it's unfinished (but to finish it you'd need to assert the first part)

Quickly grab a line or some visual range, `:'<,'>SnipRun` it and... that's it!

By selecting a visual range (always rounded line-wise) or positioning yourself on a particular line of code, and running the `SnipRun` command on it (I advise to map it), you can send the line(s) to Sniprun. Sniprun will then:

- Optionnaly, get additional information if necessary (auto retrieve import when supported for example)
- Add the boilerplate when it exists. In C, it surrounds your snip with "int main() {", "}".
- Build (write to a script file, or compile) the code
- Execute the code
- Return stdout, or stderr

## A quick word on REPL-like behavior

Some languages, see support [table](README.md#support-levels-and-languages), also have some kind of (real, or 'simulated') REPL behavior: you can expect your successive commands to behave like in a REPL interpreter, and to have 'memory' of lines you have previously sniprun'd.

Compiled languages can have this simulated REPL behavior too, though there might be unavoidable side effects.

Interpreted languages may use a simulated or real REPL, depending on the implementation.

## Installation

### Prerequisites && dependencies

- Sniprun is Linux-only for now (as of v0.4.0)
- Neovim version >= 0.44 preferably, but should work with older version
- [recommended, but optionnal] cargo and the rust toolchain version >= 1.43.0 (you can find those [here](https://www.rust-lang.org/tools/install)).
- Compiler / interpreter for the languages you work with must be installed & on your \$PATH. In case specific build tools or softwares are required, those are documented in the [doc](https://github.com/michaelb/sniprun/tree/master/doc) folder, for each interpreter, which I urge you to get a look at before getting started as it also contains the potential limitations of each interpreter.

For example, most people will probably need:

- the klepto package: `pip install --user klepto` if they use python with REPL. (Python REPL behaviour is enabled by default, but klepto has to be manually installed)

### Install Sniprun

(Recommended)

Use your favorite plugin manager.
(Run `install.sh` as a post-installation script, it will download or compile the sniprun binary)

For example, `vim-plug`:

```vim
Plug 'michaelb/sniprun', {'do': 'bash install.sh'}
" 'bash install.sh 1' to get the bleeding edge, but you'll compile sniprun at every update
```

Sniprun is developped and maintained on Linux (-only for now), support for other platforms is not in my goals plans, though simple compatibility patches PR are welcome.

## Usage

You can do basically two things: **run** (your code selection) and **stop** it (in the rare occasions it crashes, it takes too long or sniprun crashes). You'll probably be using only the first one, but the second can come in handy.

### Running

Line mode: Place your cursor on the line you want to run, and type (in command mode):

```vim
:SnipRun

```

Bloc mode: Select the code you want to execute in visual mode and type in:

```vim
:'<,'>SnipRun
```

(nota bene: the `:'<,'>` is often pre-typed and appears if you type in `:`)

### Stopping

_ARGHHH_ I Sniprun'd an infinite loop (or anything that takes too long)!
No worries, the second and last command will kill everything Sniprun ran so far (and has not finished yet):

```vim
 :SnipReset
```

Under the hood, what it does is just kill Sniprun (and its child processes) and relaunch it, and also cleans the cache directory. `:SnipReset` is the hardest 'reset' you can throw on Sniprun to vent your frustration that's hopefully not due to the plugin.
`:SnipTerminate` does the same thing but does not cleans the cache directory. (For faster recompiles, unless some data leftover in the cache was in fact the cause of the crash)

Alternatively, exit & re-enter Neovim.

### REPL-like behavior

All languages, including compiled ones, can be fitted with this (fake) REPL-like behavior.
For many languages that have an interpreter already available, a real one can be used.

Many interpreted languages will have this behavior enabled by default, but you can always disable those (or enable them) with the `g:SnipRun_repl_behavior_disable` and `g:SnipRun_repl_behavior_enable` blocklist / allowlist:

```vimrc
let g:SnipRun_repl_behavior_disable = ["Bash_original"]
let g:SnipRun_repl_behavior_enable = ["Rust_original", "Lua_original"]
```

REPL-like behavior is experimental and will work better with interpreted languages and with side-effect-free code (including prints in functions).

Hopefully, if it does not work, or if the 'memory' is corrupted by bad code (for example, in C you can't define the same function twice), you can clear the REPL memory with `:SnipReplMemoryClean` that is a faster and less error-prone alternative to `:SnipReset` for this use case.

### Configuration

You can add interpreters you want to always use in case multiples interpreters are available for one file type by adding to your config file / init.vim :

`let g:SnipRun_select_interpreters = ['name_of_the_interpreter']`

For example to always select Lua_original and Rust_original over others,

`let g:SnipRun_select_interpreters =['Lua_original', 'Rust_original']`

A list of all available interpreters can be displayed by running `:SnipList`

### My usage recommandation & tricks

- Map the run command to a simple command such as `ff` (or just `f` in visual mode).

```
nnoremap <leader>f :SnipRun<CR>
vnoremap f :SnipRun<CR>
```

- For interpreted languages with simple output, `:%SnipRun` (or a shortcut) may be a more convenient way to run your entire code.
- If you use the REPL-like behavior for some languages, mapping the repl reset to a short command is advised.

```
nnoremap <leader>c :SnipReplMemoryClean<CR>
```


## Support levels and languages

As of writing, languages can be supported up to different extents:

- **Unsupported**/**Untested** : You should not expect anything to work, except if the generic interpreter works correctly with it (at most Line level support).
- **Line** : Code contained in a single line works, for example: `print([x**2 for x in range(10)])` . Won't work if you use a variable defined elsewhere.
- **Bloc** : You can select any piece of code that is semantically correct (minus the eventual entry point) on its own (independently of indentation) in visual mode, and run it. A sniprun-able example, in Rust:

```
fn have_two() -> u16 {
  return 2;
}
let i = std::iter::once(have_two() * 3).map(|u| u*u).next().unwrap();
println!("hello n° {}", i+1);
```

- **Import** : Support external imports, so you don't have to select the top-of-file import to test a 'bloc-mode-style' code selection somewhere else.
- **File** : Sniprun will recursively find the missing variable and function definitions to run your line of code(you don't have to select a bloc anymore).
- **Project** : Sniprun will detect the root of your project, and get the necessary code from files in your project.
- **System** : Sniprun will use local (and system) libraries, such as jar files, to run your what you want.

| Language     | Support level |     | Language   | Support level    |
| ------------ | ------------- | --- | ---------- | ---------------- |
| Ada          | Untested      |     | Java       | Bloc             |
| Bash/Shell   | Bloc + REPL\* |     | JavaScript | Bloc             |
| C            | Bloc          |     | Julia      | Bloc             |
| C++          | Bloc          |     | Lisp       | Untested         |
| Clojure      | Untested      |     | Lua        | Bloc             |
| COBOL        | Untested      |     | Lua-nvim   | Bloc             |
| Coffeescript | Bloc          |     | OCaml      | Untested         |
| C#           | Untested      |     | Perl6      | Line             |
| D            | Bloc          |     | Perl       | Line             |
| Elixir       | Untested      |     | PHP        | Untested         |
| Elm          | Untested      |     | Python3    | Import +REPL\*\* |
| Erlang       | Untested      |     | R          | Bloc + REPL \*\* |
| F#           | Untested      |     | Ruby       | Bloc             |
| Go           | Bloc          |     | Rust       | Bloc             |
| Groovy       | Untested      |     | Scala      | Untested         |
| Haskell      | Bloc          |     | Scilab     | Untested         |
| Idris        | Untested      |     | Swift      | Untested         |

Want support for your language? Submit a feature request, or even better, [contribute](CONTRIBUTING.md), it's easy!

\* (fake) REPL-like functionnality, with potential unwanted side-effects

\*\* True REPL under the hood

## Known limitations

Due to its nature, Sniprun may have trouble with programs that :

- Meddle with standart output / stderr
- Need to read from stdin
- Prints double quotes ("), or incorrect UTF8 characters, or just too many lines
- Access files; sniprun does not run in a virtual environment, it accesses files just like your own code do, but since it does not run the whole program, something might go wrong. **Relative paths may cause issues**, as the current working directory for neovim won't necessarily be the one from where the binary runs, or the good one for relative imports.
- For import support level and higher, Sniprun fetch code from the saved file (and not the neovim buffer). Be sure that the functions / imports your code need have been _saved_.

## Contribute

It's super easy: see [contributing](CONTRIBUTING.md)

## Related projects

This project: [vscode-code-runner](https://github.com/formulahendry/vscode-code-runner) but sniprun is an attempt to make the same kind of plugin for Neovim, and more feature-complete. Actually, it already is (more complete, more extendable).

All [quickrun](https://github.com/thinca/vim-quickrun/blob/master/autoload/quickrun.vim) derivatives, but they are all different in the way they always all execute your entire file.

The [replvim](https://gitlab.com/HiPhish/repl.nvim) project, [vim-ipython-cell](https://github.com/hanschen/vim-ipython-cell) [codi](https://github.com/metakirby5/codi.vim) as well as [neoterm](https://github.com/kassio/neoterm) can also be used in such a way, though they are only working with languages that have a REPL.

[vimcmdline](https://github.com/jalvesaq/vimcmdline) is a close contender and so is [vim-slime](https://github.com/jpalardy/vim-slime), but they do things differently enough I made sniprun instead.

**Why should you use sniprun instead of these alternatives?**

- All-language support. Sniprun can work with virtually any language, including compiled ones. If the language is not supported yet, anyone can create a sniprun interpreter for it!
- Simpler user input & output. Sniprun doesn't use precious screen space (like [codi](https://github.com/metakirby5/codi.vim) or [vim-slime](https://github.com/jpalardy/vim-slime)).
- Promising evolution of the project: treesitter usage is in the goals plan, to make testing/ running even better (with things like auto-fecthing variables & functions definitions). Those will comply at least with the File support level for a truly amazing experience. (I'll need some help with that though).
- Fast, extendable and maintainable: this is not a 2k-lines vim script. It's a Rust project designed to be as clear and "contribuable" as possible.
