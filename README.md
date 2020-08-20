# Sniprun

Sniprun is a (still WIP) code runner plugin. It aims to provide stupidly fast testing for interpreted _and_ compiled languages.

Ever dreamt of printing the type of that obscure object, or that list to check if it contains everything you expect, but it was all pipe dream as your code would not even compile/run in its unfinished state?
Quickly grab some visual range, `:'<,'>SnipRun` it and... that's it! And there's more!

## Installation

(Recommended) Use your favorite plugin manager. (Run `install.sh` as a post-installation script, it will download or compile the sniprun binary)

For example, `vim-plug`:

```vim
Plug 'michaelb/sniprun', {'do': 'bash install.sh'}
```

Sniprun is developped and maintained on Linux (-only for now), support for other platforms is not in my goals plans, though simple compatibility patches PR are welcome.

## Usage

Sniprun is and will always (try to) be dead simple.

Line mode: Place your cursor on the line you want to run, and type (in command mode):

```vim
:SnipRun

```

Bloc mode: Select the code you want to execute in visual mode and type in:

```vim
:'<,'>SnipRun
```

(nota bene: the `:'<,'>` is often pre-typed)
_ARGHHH_ I 'SnipRan' and infinite loop (or anything that takes too long)!
No worries, the second and last command will kill everything Sniprun ran so far (and has not finished yet):

```vim
 :SnipTerminate
```

Alternatively, exit Neovim.

### My usage recommandation

- Map the line mode to a simple command such as `ff`
- Rename `SnipRun` to a more convenient command that do not conflict with your existing mappings

## Dump

If the code needed to run your line/bloc is not completely contained within your line or selection, depending on the support level for your language, sniprun will automatically fetch it from the current file, the current project or even use your project's dependencies and system libraries.
