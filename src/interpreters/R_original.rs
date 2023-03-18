#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct R_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    r_work_dir: String,
    main_file_path: String,
}
impl Interpreter for R_original {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<R_original> {
        let bwd = data.work_dir.clone() + "/R-original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&bwd)
            .expect("Could not create directory for R-original");
        let mfp = bwd.clone() + "/main.r";
        Box::new(R_original {
            data,
            support_level: level,
            code: String::from(""),
            r_work_dir: bwd,
            main_file_path: mfp,
        })
    }

    fn get_name() -> String {
        String::from("R_original")
    }

    fn get_supported_languages() -> Vec<String> {
        vec![String::from("R"), String::from("r")]
    }
    fn behave_repl_like_default() -> bool {
        true
    }
    fn has_repl_capability() -> bool {
        true
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

    fn check_cli_args(&self) -> Result<(), SniprunError> {
        // All cli arguments are sendable to python
        // Though they will be ignored in REPL mode
        Ok(())
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
            File::create(&self.main_file_path).expect("Failed to create file for R-original");

        write(&self.main_file_path, &self.code).expect("Unable to write to file for R-original");
        Ok(())
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        let output = Command::new("Rscript")
            .arg(&self.main_file_path)
            .args(&self.get_data().cli_args)
            .output()
            .expect("Unable to start process");
        info!("yay from R interpreter");
        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else if R_original::error_truncate(&self.get_data()) == ErrTruncate::Short {
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
impl ReplLikeInterpreter for R_original {
    fn fetch_code_repl(&mut self) -> Result<(), SniprunError> {
        self.fetch_code()
    }
    fn build_repl(&mut self) -> Result<(), SniprunError> {
        self.build()
    }
    fn execute_repl(&mut self) -> Result<String, SniprunError> {
        self.execute()
    }

    fn add_boilerplate_repl(&mut self) -> Result<(), SniprunError> {
        info!("repl mode");
        let mut final_code = String::new();

        let rdata_path = self.r_work_dir.clone() + "/sniprun.RData";

        if self.read_previous_code().is_empty() {
            //first run
            self.save_code(String::from("Not the first R run anymore"));
        } else {
            // not first run, tell R to load old variables
            {
                final_code.push_str("load('");
                final_code.push_str(&rdata_path);
                final_code.push_str("')");
            }
        }
        final_code.push('\n');
        final_code.push_str(&self.code);
        final_code.push('\n');

        {
            //save state
            final_code.push_str("save.image('");
            final_code.push_str(&rdata_path);
            final_code.push_str("')");
        }
        self.code = final_code;
        Ok(())
    }
}

#[cfg(test)]
mod test_r_original {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial(r_original)]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("print(\"Hi\");");
        let mut interpreter = R_original::new(data);
        let res = interpreter.run();

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert!(string_result.contains("Hi"));
    }
}
