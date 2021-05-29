// Be sure to read the CONTRIBUTING.md file :-)

#[derive(Clone)]
#[allow(non_camel_case_types)]
// For example, Rust_original is a good name for the first rust interpreter
pub struct Mathematica_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,

    language_work_dir: String,
    main_file_path: String,
}

//necessary boilerplate, you don't need to implement that if you want a Bloc support level
//interpreter (the easiest && most common)
impl ReplLikeInterpreter for Mathematica_original {}

impl Interpreter for Mathematica_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Mathematica_original> {
        //create a subfolder in the cache folder
        let lwd = data.work_dir.clone() + "/mathematica_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&lwd)
            .expect("Could not create directory for example");

        //pre-create string pointing to main file's and binary's path
        let mfp = lwd.clone() + "/main.mma";
        Box::new(Mathematica_original {
            data,
            support_level,
            code: String::new(),
            language_work_dir: lwd,
            main_file_path: mfp,
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("Mathematica"),
            String::from("mathematica"),
            String::from("mma"),
        ]
    }

    fn get_name() -> String {
        String::from("Mathematica_original")
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
        SupportLevel::Bloc
    }

    fn fetch_code(&mut self) -> Result<(), SniprunError> {
        if !self
            .data
            .current_bloc
            .replace(&[' ', '\t', '\n', '\r'][..], "")
            .is_empty()
            && self.support_level >= SupportLevel::Bloc
        {
            self.code = self.data.current_bloc.clone();
        } else if !self.data.current_line.replace(" ", "").is_empty()
            && self.support_level >= SupportLevel::Line
        {
            self.code = self.data.current_line.clone();
        } else {
            self.code = String::from("");
        }
        Ok(())
    }

    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        self.code = String::from("") + &self.code + "Exit[]";
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        //write code to file
        let mut _file = File::create(&self.main_file_path)
            .expect("Failed to create file for mathematica_original");
        // IO errors can be ignored, or handled into a proper SniprunError
        // If you panic, it should not be too dangerous for anyone
        write(&self.main_file_path, &self.code)
            .expect("Unable to write to file for mathematica_original");

        Ok(())
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        //run th binary and get the std output (or stderr)
        let output = Command::new("wolfram")
            .arg("-noprompt")
            .arg("-script")
            .arg(&self.main_file_path)
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
