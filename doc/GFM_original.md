the GFM_original (Github flavored markdown) can help run code blocs embedded in markdown.


```bash
echo "lol"  # << you can run sniprun on this line
```

```rust  << running on this line is undefined behavior
println!("test");
``` 


the language name must be there at the bloc start and has to match the github flavor syntax, and the underlying interpreter must be callable (no missing compiler etc...)
