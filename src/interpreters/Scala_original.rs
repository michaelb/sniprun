#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Scala_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,

    ///specific to compiled languages, can be modified of course
    language_work_dir: String,
    main_file_path: String,
    // you can and should add fields as needed
}

//necessary boilerplate, you don't need to implement that if you want a Bloc support level
//interpreter (the easiest && most common)
impl ReplLikeInterpreter for Scala_original {}

impl Interpreter for Scala_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Scala_original> {
        //create a subfolder in the cache folder
        let lwd = data.work_dir.clone() + "/scala_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&lwd)
            .expect("Could not create directory for example");

        //pre-create string pointing to main file's and binary's path
        let mfp = lwd.clone() + "/Main.scala";
        Box::new(Scala_original {
            data,
            support_level,
            code: String::new(),
            language_work_dir: lwd,
            main_file_path: mfp,
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("Scala"), // in 1st position of vector, used for info only
            String::from("scala"),
        ]
    }

    fn get_name() -> String {
        String::from("Scala_original")
    }

    fn default_for_filetype() -> bool {
        true
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
            // if bloc is not pseudo empty and has Bloc current support level,
            // add fetched code to self
            self.code = self.data.current_bloc.clone();

        // if there is only data on current line / or Line is the max support level
        } else if !self.data.current_line.replace(' ', "").is_empty()
            && self.support_level >= SupportLevel::Line
        {
            self.code = self.data.current_line.clone();
        } else {
            // no code was retrieved
            self.code = String::from("");
        }
        info!("scala interpreter fetched code");
        Ok(())
    }

    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        // an example following Rust's syntax

        if !Scala_original::contains_main("int main (", &self.code, "//") {
            self.code = String::from("object Main {\ndef main(arg: Array[String]) = {")
                + &self.code
                + "}\n}";
        }
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for language_subname");
        // IO errors can be ignored, or handled into a proper SniprunError
        // If you panic, it should not be too dangerous for anyone
        write(&self.main_file_path, &self.code)
            .expect("Unable to write to file for language_subname");

        //compile it (to the bin_path that arleady points to the rigth path)
        let output = Command::new("scalac")
            .arg("-d")
            .arg(&self.language_work_dir)
            .arg(&self.main_file_path)
            .output()
            .expect("Unable to start process");

        // if relevant, return the error number (parse it from stderr)
        if !output.status.success() {
            Err(SniprunError::CompilationError(
                String::from_utf8(output.stderr).unwrap(),
            ))
        } else {
            info!("scala compiled successfully");
            Ok(())
        }
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        //run th binary and get the std output (or stderr)
        let output = Command::new("scala")
            .arg("Main")
            .current_dir(&self.language_work_dir)
            .output()
            .expect("Unable to start process");

        if output.status.success() {
            //return stdout
            Ok(String::from_utf8(output.stdout).unwrap())
        } else if Scala_original::error_truncate(&self.get_data()) == ErrTruncate::Short {
            Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr.clone())
                    .unwrap()
                    .lines()
                    .next()
                    .unwrap_or(&String::from_utf8(output.stderr).unwrap())
                    .to_owned(),
            ))
        } else {
            Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr).unwrap(),
            ))
        }
    }
}

#[cfg(test)]
mod test_scala_original {
    use super::*;
    #[test]
    fn simple_print() {
        let mut data = DataHolder::new();

        //inspired from Rust syntax
        data.current_bloc = String::from("println(\"Hi\")");
        let mut interpreter = Scala_original::new(data);
        let res = interpreter.run();

        let string_result = res.unwrap();

        // -> compare result with predicted
        assert_eq!(string_result, "Hi\n");
    }
}
