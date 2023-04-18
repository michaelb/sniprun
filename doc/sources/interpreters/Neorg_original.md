## Neorg original

the Neorg\_original interpreter helps you running code blocs defined in neorg code blocs delimiters

inline, switches and headers are not supported/ignored

### example 1


```
#name demo
@code bash

echo "lol"  # << you can run sniprun on this line



\# or the whole visual selection following:

for i in {1..4};do

echo $i

done
@end

```


### example 2


```
#name demo_run_whole_bloc         << running on this line or the line below will run the entire bloc
@code rust                        

println!("test");
println!("test2");
@end
```


Even though it is possible to have empty lines in between the #name tag and the @code block for this plugin this doesn't work. The #name has to be in the line directly above the @code block

```
#name name_tag_not_working        << this #name tag doesn't run the code below 


@code rust                        

println!("test");
println!("test2");
@end

``` 


**the language name must be there (otherwise the default * will be used) at the bloc start** and has to match the language name or the filetype associated 

\* python, but you can ofc configure that: 

```
require'sniprun'.setup({
    interpreter_options = {
        Neorg_original = { 
            default_filetype = 'bash' -- default filetype/language name
        }
    }
})
```

### example 3: running named code blocs

```
#name mycodebloc 
@code rust                        
println!("test");
@end
```

running `:%SnipRun mycodebloc` will run this code bloc (and any code bloc named similarly, case-insensitively)

running `:%SnipRun` without any further arguments will run all the code blocs
