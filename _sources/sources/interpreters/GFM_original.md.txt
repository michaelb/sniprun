## GFM original (Markdown)

the GFM_original (Github flavored markdown) can help run code blocs embedded in markdown.

### example 1


\```bash

echo "lol"  # << you can run sniprun on this line



\# or the whole visual selection following:

for i in {1..4};do

echo $i

done

\```


### example 2


\``rust  << running on this line will run the entire bloc

println!("test");

\``` 


**the language name must be there (otherwise the default * will be used) at the bloc start** and has to match the github flavor syntax, and the underlying interpreter must be callable (no missing compiler etc...)

\* python, but you can ofc configure that: 

```
require'sniprun'.setup({
    interpreter_options = {
        GFM_original = { 
            default_filetype = 'bash' -- default filetype (not github flavored markdown name)
        }
    }
})
```


if GFM doesn't include a flavor that matches the language you want, well it's not really GFM anymore but you can but the filetype of the language you want, such as julia or lua




