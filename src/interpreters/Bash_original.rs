use crate::interpreters::import::*;

#[allow(non_camel_case_types)]
pub struct Bash_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    main_file_path: String,
}

impl ReplLikeInterpreter for Bash_original {}
impl Interpreter for Bash_original {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<Bash_original> {
        let bwd = data.work_dir.clone() + "/bash-original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&bwd)
            .expect("Could not create directory for bash-original");
        let mfp = bwd + "/main.sh";
        Box::new(Bash_original {
            data,
            support_level: level,
            code: String::from(""),
            main_file_path: mfp,
        })
    }

    fn get_name() -> String {
        String::from("Bash_original")
    }

    fn behave_repl_like_default() -> bool {
        false
    }
    fn has_repl_capability() -> bool {
        true
    }

    fn default_for_filetype() -> bool {
        true
    }

    fn check_cli_args(&self) -> Result<(), SniprunError> {
        // All cli arguments are sendable to python
        // Though they will be ignored in REPL mode
        Ok(())
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("Bash / Shell"),
            String::from("bash"),
            String::from("shell"),
            String::from("sh"),
        ]
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
            && self.get_current_level() >= SupportLevel::Bloc
        {
            self.code.clone_from(&self.data.current_bloc);
        } else if !self.data.current_line.replace(' ', "").is_empty()
            && self.get_current_level() >= SupportLevel::Line
        {
            self.code.clone_from(&self.data.current_line);
        } else {
            self.code = String::from("");
        }
        Ok(())
    }

    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        //add shebang just in case
        self.code = String::from("#!/usr/bin/env bash \n")
            + "sniprun_main123456789(){\n"
            + &self.code
            + "\n}\n"
            + "sniprun_main123456789\n";
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for bash-original");

        write(&self.main_file_path, &self.code).expect("Unable to write to file for bash-original");
        Ok(())
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        let interpreter = Bash_original::get_interpreter_or(&self.data, "bash");
        let output = Command::new(interpreter.split_whitespace().next().unwrap())
            .args(interpreter.split_whitespace().skip(1))
            .arg(&self.main_file_path)
            .args(&self.get_data().cli_args)
            .output()
            .expect("Unable to start process");
        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else if Bash_original::error_truncate(&self.get_data()) == ErrTruncate::Short {
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
mod test_bash_original {
    use super::*;

    #[test]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("A=2 && echo $A");
        let mut interpreter = Bash_original::new(data);
        let res = interpreter.run();

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert_eq!(string_result, "2\n");
    }
}
