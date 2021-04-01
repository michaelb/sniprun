the GFM_original (Github flavored markdown) can help run code blocs embedded in markdown.

# example 1

-------------

\```bash

echo "lol"  # << you can run sniprun on this line



\# or the whole visual selection following:

for i in {1..4};do

echo $i

done

\```


# example 2

-------------

\``rust  << running on this line will run the entire bloc

println!("test");

\``` 

-------------

**the language name must be there at the bloc start** and has to match the github flavor syntax, and the underlying interpreter must be callable (no missing compiler etc...)



