// Be sure to read the CONTRIBUTING.md file :-)

#[derive(Clone)]
#[allow(non_camel_case_types)]
// For example, Rust_original is a good name for the first rust interpreter
pub struct Language_subname {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,

    ///specific to compiled languages, can be modified of course
    language_work_dir: String,
    bin_path: String,
    main_file_path: String,
    // you can and should add fields as needed
}

//necessary boilerplate, you don't need to implement that if you want a Bloc support level
//interpreter (the easiest && most common)
impl ReplLikeInterpreter for Language_subname {}

impl Interpreter for Language_subname {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Language_subname> {
        //create a subfolder in the cache folder
        let lwd = data.work_dir.clone() + "/language_subname";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&lwd)
            .expect("Could not create directory for example");

        //pre-create string pointing to main file's and binary's path
        let mfp = lwd.clone() + "/main.extension";
        let bp = lwd.clone() + "/main"; // remove extension so binary is named 'main'
        Box::new(Language_subname {
            data,
            support_level,
            code: String::new(),
            language_work_dir: lwd,
            bin_path: bp,
            main_file_path: mfp,
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("Official language name"), // in 1st position of vector, used for info only
            //':set ft?' in nvim to get the filetype of opened file
            String::from("language_filetype"),
            String::from("extension"), //should not be necessary, but just in case
                                       // another similar name (like python and python3)?
        ]

        // little explanation: only the filetype is necessary, but the 1st element of the Vec is
        // displayed with SnipInfo, so put "JavaScript" instead of "js" for clarity's sake
    }

    fn get_name() -> String {
        // get your interpreter name
        String::from("Language_subname")
    }

    fn get_current_level(&self) -> SupportLevel {
        self.support_level
    }
    fn set_current_level(&mut self, level: SupportLevel) {
        self.support_level = level;
    }

    fn get_data(&self) -> DataHolder {
        self.data.clone()
    }

    fn get_max_support_level() -> SupportLevel {
        //define the max level support of the interpreter (see readme for definitions)
        SupportLevel::Bloc
    }

    fn fetch_code(&mut self) -> Result<(), SniprunError> {
        //note: you probably don't have to modify, or even understand this function

        //here if you detect conditions that make higher support level impossible,
        //or unecessary, you should set the current level down. Then you will be able to
        //ignore maybe-heavy code that won't be needed anyway

        //add code from data to self.code
        if !self
            .data
            .current_bloc
            .replace(&[' ', '\t', '\n', '\r'][..], "")
            .is_empty()
            && self.support_level >= SupportLevel::Bloc
        {
            // if bloc is not pseudo empty and has Bloc current support level,
            // add fetched code to self
            self.code = self.data.current_bloc.clone();

        // if there is only data on current line / or Line is the max support level
        } else if !self.data.current_line.replace(" ", "").is_empty()
            && self.support_level >= SupportLevel::Line
        {
            self.code = self.data.current_line.clone();
        } else {
            // no code was retrieved
            self.code = String::from("");
        }

        // now self.code contains the line or bloc of code wanted :-)
        Ok(())
    }

    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        // an example following Rust's syntax
        self.code = String::from("fn main() {") + &self.code + "}";
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        //write code to file
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for language_subname");
        // IO errors can be ignored, or handled into a proper SniprunError
        // If you panic, it should not be too dangerous for anyone
        write(&self.main_file_path, &self.code)
            .expect("Unable to write to file for language_subname");


        //fetch the option from the configuration
        //  interpreter_options = {
        //   example_original = {
        //     example_option = "--optimize-with-debug-info",
        //   }
        // },

        let mut configurable_option = String::from("--optimize"); // no debug info by default, for example

        if let Some(config_value) = self.get_interpreter_option("example_option") {
            if let Some(config_value_valid_string) = config_value.as_str() {
                configurable_option = config_value_valid_string.to_string();
            }
        }



        //compile it (to the bin_path that arleady points to the rigth path)
        let output = Command::new("compiler")
            .arg(&configurable_option) // for short snippets, that may contain a long loop
            .arg("--out-dir")
            .arg(&self.language_work_dir)
            .arg(&self.main_file_path)
            .output()
            .expect("Unable to start process");

        // if relevant, return the error number (parse it from stderr)
        if !output.status.success() {
            return Err(SniprunError::CompilationError(
                "some relevant feedback".to_string(), //such as output.stderr :-)
            ));
        } else {
            return Ok(());
        }
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        //run th binary and get the std output (or stderr)
        let output = Command::new(&self.bin_path)
            .output()
            .expect("Unable to start process");

        if output.status.success() {
            //return stdout
            return Ok(String::from_utf8(output.stdout).unwrap());
        } else {
            // return stderr
            return Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr).unwrap(),
            ));
        }
    }
}

// You can add tests if you want to
#[cfg(test)]
mod test_language_subname {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial] // multiple test of the same interpreter should run sequentially:
              // after all, they write and read to the same dir/files
    fn simple_print() {
        let mut data = DataHolder::new();

        //inspired from Rust syntax
        data.current_bloc = String::from("println!(\"HW, 1+1 = {}\", 1+1)");
        let mut interpreter = Language_subname::new(data);
        let res = interpreter.run();

        // -> should panic if not an Ok()
        // let string_result = res.unwrap();

        // -> compare result with predicted
        // assert_eq!(string_result, "HW, 1+1 = 2\n");
    }

    #[test]
    #[serial]
    fn another_test() {
        //another test
    }
}
