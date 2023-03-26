#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Ruby_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    main_file_path: String,
}
impl ReplLikeInterpreter for Ruby_original {}
impl Interpreter for Ruby_original {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<Ruby_original> {
        let bwd = data.work_dir.clone() + "/ruby-original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&bwd)
            .expect("Could not create directory for ruby-original");
        let mfp = bwd + "/main.rb";
        Box::new(Ruby_original {
            data,
            support_level: level,
            code: String::from(""),
            main_file_path: mfp,
        })
    }

    fn get_name() -> String {
        String::from("Ruby_original")
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("Ruby"),
            String::from("ruby"),
            String::from("rb"),
        ]
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

    fn check_cli_args(&self) -> Result<(), SniprunError> {
        // All cli arguments are sendable to python
        // Though they will be ignored in REPL mode
        Ok(())
    }

    fn fetch_code(&mut self) -> Result<(), SniprunError> {
        if !self
            .data
            .current_bloc
            .replace(&[' ', '\t', '\n', '\r'][..], "")
            .is_empty()
            && self.get_current_level() >= SupportLevel::Bloc
        {
            self.code = self.data.current_bloc.clone();
        } else if !self.data.current_line.replace(' ', "").is_empty()
            && self.get_current_level() >= SupportLevel::Line
        {
            self.code = self.data.current_line.clone();
        } else {
            self.code = String::from("");
        }
        Ok(())
    }

    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for ruby-original");

        write(&self.main_file_path, &self.code).expect("Unable to write to file for ruby-original");
        Ok(())
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        let output = Command::new("ruby")
            .arg(&self.main_file_path)
            .args(&self.get_data().cli_args)
            .output()
            .expect("Unable to start process");
        info!("yay from ruby interpreter");
        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else if Ruby_original::error_truncate(&self.get_data()) == ErrTruncate::Short {
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
mod test_ruby_original {
    use super::*;

    #[test]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("puts \"hell\"");
        let mut interpreter = Ruby_original::new(data);
        let res = interpreter.run();

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert_eq!(string_result, "hell\n");
    }
}
