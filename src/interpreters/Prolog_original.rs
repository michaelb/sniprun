#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Prolog_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    main_file_path: String,
    interpreter: String,
}
impl ReplLikeInterpreter for Prolog_original {}
impl Interpreter for Prolog_original {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<Prolog_original> {
        let bwd = data.work_dir.clone() + "/prolog-original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&bwd)
            .expect("Could not create directory for prolog-original");
        let mfp = bwd + "/main.pl";
        Box::new(Prolog_original {
            data,
            support_level: level,
            code: String::from(""),
            main_file_path: mfp,
            interpreter: String::new(),
        })
    }
    fn get_name() -> String {
        String::from("Prolog_original")
    }
    fn get_supported_languages() -> Vec<String> {
        vec![String::from("Prolog"), String::from("prolog")]
    }
    fn get_current_level(&self) -> SupportLevel {
        self.support_level
    }
    fn set_current_level(&mut self, level: SupportLevel) {
        self.support_level = level
    }
    fn default_for_filetype() -> bool {
        true
    }
    fn get_data(&self) -> DataHolder {
        self.data.clone()
    }
    fn check_cli_args(&self) -> Result<(), SniprunError> {
        // All cli arguments are sendable to the exe
        Ok(())
    }
    fn get_max_support_level() -> SupportLevel {
        SupportLevel::Bloc
    }
    fn fetch_code(&mut self) -> Result<(), SniprunError> {
        let default_interpreter = String::from("gprolog");
        self.interpreter = default_interpreter;
        if let Some(used_interpreter) =
            Python3_fifo::get_interpreter_option(&self.get_data(), "interpreter")
        {
            if let Some(interpreter_string) = used_interpreter.as_str() {
                info!("Using custom interpreter: {}", interpreter_string);
                self.interpreter = interpreter_string.to_string();
            }
        }

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
            File::create(&self.main_file_path).expect("Failed to create file for prolog-original");

        write(&self.main_file_path, &self.code)
            .expect("Unable to write to file for prolog-original");
        Ok(())
    }
    fn execute(&mut self) -> Result<String, SniprunError> {
        let output = if self.interpreter != "gprolog" {
            Command::new(self.interpreter.clone())
                .arg(&self.main_file_path)
                .args(&self.get_data().cli_args)
                .output()
                .expect("Unable to start process")
        } else {
            // special case for gprolog which needs the --consult-file arg
            Command::new("gprolog")
                .arg(String::from("--consult-file"))
                .arg(&self.main_file_path)
                .args(&self.get_data().cli_args)
                .output()
                .expect("Unable to start process")
        };
        info!("yay from Prolog interpreter");
        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else if Prolog_original::error_truncate(&self.get_data()) == ErrTruncate::Short {
            return Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr.clone())
                    .unwrap()
                    .lines()
                    .last()
                    .unwrap_or(&String::from_utf8(output.stderr).unwrap())
                    .to_owned(),
            ));
        } else {
            return Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr).unwrap(),
            ));
        }
    }
}
#[cfg(test)]
mod test_prolog_original {
    use super::*;

    // #[test]
    #[allow(dead_code)]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from(":- write(ok), halt.");
        let mut interpreter = Prolog_original::new(data);
        let res = interpreter.run();

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert_eq!(string_result, "ok");
    }
}
