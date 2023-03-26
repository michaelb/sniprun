#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct D_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,

    main_file_path: String,
}
impl ReplLikeInterpreter for D_original {}
impl Interpreter for D_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<D_original> {
        //create a subfolder in the cache folder
        let rwd = data.work_dir.clone() + "/d_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&rwd)
            .expect("Could not create directory for d-original");

        //pre-create string pointing to main file's and binary's path
        let mfp = rwd + "/main.d";
        Box::new(D_original {
            data,
            support_level,
            code: String::from(""),
            main_file_path: mfp,
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![String::from("D"), String::from("d"), String::from("dlang")]
    }

    fn get_name() -> String {
        String::from("D_original")
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
        SupportLevel::Bloc
    }

    fn fetch_code(&mut self) -> Result<(), SniprunError> {
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
            self.code = String::from("");
        }
        Ok(())
    }

    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        self.code = String::from("import std.stdio;\nvoid main() {") + &self.code + "}";
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        //write code to file
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for d-original");
        write(&self.main_file_path, &self.code).expect("Unable to write to file for d-original");
        Ok(())
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        //run th binary and get the std output (or stderr)
        let output = Command::new("dmd")
            .arg("-run")
            .arg(&self.main_file_path)
            .output()
            .expect("Unable to start process");
        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else if D_original::error_truncate(&self.get_data()) == ErrTruncate::Short {
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
mod test_d_original {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial(d_original)]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc =
            String::from("string yourName = \"a\";\nwritefln(\"Hi %s!\", yourName);");
        let mut interpreter = D_original::new(data);
        let res = interpreter.run();

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert_eq!(string_result, "Hi a!\n");
    }
}
