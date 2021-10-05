#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Go_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,

    ///specific to go
    compiler: String,
    go_work_dir: String,
    bin_path: String,
    main_file_path: String,
}

impl Go_original {
    fn fetch_config(&mut self) {
        let default_compiler = String::from("go");
        if let Some(used_compiler) = self.get_interpreter_option("compiler") {
            if let Some(compiler_string) = used_compiler.as_str() {
                info!("Using custom compiler: {}", compiler_string);
                self.compiler = compiler_string.to_string();
            }
        }
        self.compiler = default_compiler;
    }
}


impl ReplLikeInterpreter for Go_original {}
impl Interpreter for Go_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Go_original> {
        //create a subfolder in the cache folder
        let gwd = data.work_dir.clone() + "/go_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&gwd)
            .expect("Could not create directory for go-original");

        //pre-create string pointing to main file's and binary's path
        let mfp = gwd.clone() + "/main.go";
        let bp = String::from(&mfp[..mfp.len() - 3]); // remove extension so binary is named 'main'
        Box::new(Go_original {
            data,
            support_level,
            code: String::from(""),
            go_work_dir: gwd,
            bin_path: bp,
            main_file_path: mfp,
            compiler: String::new(),
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("Go"),
            String::from("go"),
            String::from("golang"),
        ]
    }

    fn get_name() -> String {
        String::from("Go_original")
    }

    fn default_for_filetype() -> bool {
        true
    }

    fn check_cli_args(&self) -> Result<(), SniprunError> {
        // All cli arguments are sendable to python
        // Though they will be ignored in REPL mode
        Ok(())
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
        self.fetch_config();
        //add code from data to self.code
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
        self.code = String::from("package main \nimport \"fmt\"\nfunc main() {") + &self.code + "}";
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        //write code to file
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for go-original");
        write(&self.main_file_path, &self.code).expect("Unable to write to file for go-original");

        //compile it (to the bin_path that arleady points to the rigth path)
        let output = Command::new(&self.compiler)
            .arg("build")
            .arg("-o")
            .arg(&self.go_work_dir)
            .arg(&self.main_file_path)
            .output()
            .expect("Unable to start process");

        //TODO if relevant, return the error number (parse it from stderr)
        if !output.status.success() {
            Err(SniprunError::CompilationError("".to_string()))
        } else {
            Ok(())
        }
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        //run th binary and get the std output (or stderr)
        let output = Command::new(&self.bin_path)
            .args(&self.get_data().cli_args)
            .output()
            .expect("Unable to start process");
        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else {
            Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr).unwrap(),
            ))
        }
    }
}

#[cfg(test)]
mod test_go_original {
    use super::*;

    use serial_test::serial;

    #[test]
    #[serial(go)]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("fmt.Println(\"Hello\")");
        let mut interpreter = Go_original::new(data);
        let res = interpreter.run();

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert_eq!(string_result, "Hello\n");
    }
}
