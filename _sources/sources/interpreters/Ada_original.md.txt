## Ada original

dependencies: Need `gcc-ada` and `gnatmake`


Note: because Ada needs variables to be declared before the begin
(in a non contiguous section of the file), SnipRun is not very useful
here and will, in practice, only be able to run blocs like

```ada
Put_Line("raw text");
```
