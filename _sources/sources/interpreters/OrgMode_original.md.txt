## Orgmode original

the Orgmode\_original interpreter helps you running code blocs defined in org code blocs delimiters

inline, switches and headers are not supported/ignored

### example 1


```
#+NAME: demo
#+BEGIN_SRC bash

echo "lol"  # << you can run sniprun on this line



\# or the whole visual selection following:

for i in {1..4};do

echo $i

done
#+END_SRC

```


### example 2


```
#+NAME: demo_run_whole_bloc
#+BEGIN_SRC rust                        << running on this line will run the entire bloc

println!("test");
println!("test2");
#+END_SRC

``` 


**the language name must be there (otherwise the default * will be used) at the bloc start** and has to match the language name or the filetype associated 

\* python, but you can ofc configure that: 

```
require'sniprun'.setup({
    interpreter_options = {
        OrgMode_original = { 
            default_filetype = 'bash' -- default filetype/language name
        }
    }
})
```

### example 3: running named code blocs

```
#+NAME: mycodebloc 
#+BEGIN_SRC rust                        
println!("test");
#+END_SRC
```

running `:%SnipRun mycodebloc` will run this code bloc (and any code bloc named similarly, case-insensitively)

running `:%SnipRun` without any further arguments will run all the code blocs
