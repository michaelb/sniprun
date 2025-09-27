## Bash original

Beware of Bash_original, as it runs as script on your system,
with access to your ENV variables and PATH etc...

removing a file from absolute path will succeed!

REPL mode has also many quirks (not a true repl, will rerun
previously sniprun'd commands). Overall I strongly suggest _not_ using it
