#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct TypeScript_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    main_file_path: String,
}

impl ReplLikeInterpreter for TypeScript_original {}

impl Interpreter for TypeScript_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<TypeScript_original> {
        //create a subfolder in the cache folder
        let lwd = data.work_dir.clone() + "/typescript_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&lwd)
            .expect("Could not create directory for example");

        //pre-create string pointing to main file's and binary's path
        let mfp = lwd + "/main.ts";
        Box::new(TypeScript_original {
            data,
            support_level,
            code: String::new(),
            main_file_path: mfp,
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("TypeScript"), // in 1st position of vector, used for info only
            //':set ft?' in nvim to get the filetype of opened file
            String::from("typescript"),
            String::from("typescriptreact"),
            String::from("ts"), //should not be necessary, but just in case
                                // another similar name (like python and python3)?
        ]
    }

    fn get_name() -> String {
        // get your interpreter name
        String::from("TypeScript_original")
    }

    fn get_current_level(&self) -> SupportLevel {
        self.support_level
    }
    fn set_current_level(&mut self, level: SupportLevel) {
        self.support_level = level;
    }

    fn default_for_filetype() -> bool {
        true
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
            self.code = self.data.current_bloc.clone();
        } else if !self.data.current_line.replace(' ', "").is_empty()
            && self.support_level >= SupportLevel::Line
        {
            self.code = self.data.current_line.clone();
        } else {
            // no code was retrieved
            self.code = String::from("");
        }

        // now self.code contains the line or bloc of code wanted :-)
        info!("Typescript self.code) = {}", self.code);
        Ok(())
    }

    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        //write code to file
        let mut _file = File::create(&self.main_file_path)
            .expect("failed to create file for typescript_original");
        // io errors can be ignored, or handled into a proper sniprunerror
        // if you panic, it should not be too dangerous for anyone
        write(&self.main_file_path, &self.code)
            .expect("unable to write to file for typescript_original");

        Ok(())
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        //run th binary and get the std output (or stderr)
        let output = Command::new("ts-node")
            .arg(&self.main_file_path)
            .output()
            .expect("Unable to start process");

        if output.status.success() {
            //return stdout
            Ok(String::from_utf8(output.stdout).unwrap())
        } else if TypeScript_original::error_truncate(&self.get_data()) == ErrTruncate::Short {
            Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr.clone())
                    .unwrap()
                    .lines()
                    .filter(|l| l.contains("Error:"))
                    .last()
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

// You can add tests if you want to
#[cfg(test)]
mod test_typescript_original {
    use super::*;
    #[test]
    fn simple_print() {
        let mut data = DataHolder::new();

        //inspired from Rust syntax
        data.current_bloc = String::from("let message: string = 'Hi';\nconsole.log(message);");
        let mut interpreter = TypeScript_original::new(data);
        let res = interpreter.run();

        // -> should panic if not an Ok()
        let string_result = res.unwrap();

        // -> compare result with predicted
        assert_eq!(string_result, "Hi\n");
    }
}
