#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct C_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    bin_path: String,
    main_file_path: String,
    compiler: String,
    imports: Vec<String>,
}

impl C_original {
    fn fetch_imports(&mut self) -> Result<(), SniprunError> {
        if self.support_level < SupportLevel::Import {
            return Ok(());
        }

        let mut v = vec![];
        let mut errored = true;
        if let Some(real_nvim_instance) = self.data.nvim_instance.clone() {
            info!("got real nvim instance");
            let mut rvi = real_nvim_instance.lock().unwrap();
            if let Ok(buffer) = rvi.get_current_buf() {
                info!("got buffer");
                if let Ok(buf_lines) = buffer.get_lines(&mut rvi, 0, -1, false) {
                    info!("got lines in buffer");
                    v = buf_lines;
                    errored = false;
                }
            }
        }

        if errored {
            return Err(SniprunError::FetchCodeError);
        }

        for line in v.iter() {
            if line.starts_with("#include <") {
                self.imports.push(line.to_string());
            }
            if line.starts_with("#include")
                && (std::env::var("C_INCLUDE_PATH").is_ok()
                    || std::env::var("CPLUS_INCLUDE_PATH").is_ok())
            {
                self.imports.push(line.to_string());
            }
        }
        info!("fecthed imports : {:?}", self.imports);
        Ok(())
    }

    fn fetch_config(&mut self) {
        let default_compiler = String::from("gcc");
        self.compiler = default_compiler;
        if let Some(used_compiler) =
            C_original::get_interpreter_option(&self.get_data(), "compiler")
        {
            if let Some(compiler_string) = used_compiler.as_str() {
                info!("Using custom compiler: {}", compiler_string);
                self.compiler = compiler_string.to_string();
            }
        }
    }
}

impl ReplLikeInterpreter for C_original {}

impl Interpreter for C_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<C_original> {
        let rwd = data.work_dir.clone() + "/c_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&rwd)
            .expect("Could not create directory for c-original");
        let mfp = rwd + "/main.c";
        let bp = String::from(&mfp[..mfp.len() - 2]);
        Box::new(C_original {
            data,
            support_level,
            code: String::from(""),
            bin_path: bp,
            main_file_path: mfp,
            compiler: String::new(),
            imports: vec![],
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![String::from("C"), String::from("c")]
    }

    fn get_name() -> String {
        String::from("C_original")
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
        SupportLevel::Import
    }

    fn check_cli_args(&self) -> Result<(), SniprunError> {
        // All cli arguments are sendable to python
        // Though they will be ignored in REPL mode
        Ok(())
    }

    fn fetch_code(&mut self) -> Result<(), SniprunError> {
        self.fetch_config();
        if !self
            .data
            .current_bloc
            .replace(&[' ', '\t', '\n', '\r'][..], "")
            .is_empty()
        {
            self.code = self.data.current_bloc.clone();
        } else if !self.data.current_line.replace(' ', "").is_empty() {
            self.code = self.data.current_line.clone();
        } else {
            self.code = String::from("");
        }
        Ok(())
    }

    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        self.fetch_imports()?;

        if !C_original::contains_main("int main (", &self.code, "//") {
            self.code = String::from("int main() {\n") + &self.code + "\nreturn 0;}";
        }
        if !self.imports.iter().any(|s| s.contains("<stdio.h>")) {
            self.code = String::from("#include <stdio.h>\n") + &self.code;
        }
        self.code = self.imports.join("\n") + "\n" + &self.code;

        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        info!("starting build");
        //write code to file

        let mut build_args: Vec<String> = vec![];
        if let Ok(cflags) = std::env::var("CFLAGS") {
            info!("CFLAGS env var found : {}", cflags);
            build_args.extend(cflags.split_whitespace().map(|s| s.to_owned()));
        }

        if let Ok(c_incl_path) = std::env::var("C_INCLUDE_PATH") {
            info!("C_INCLUDE_PATH env var found : {}", c_incl_path);
            build_args.extend(c_incl_path.split(':').map(|s| String::from("-I") + s));
        }

        if let Ok(cplus_incl_path) = std::env::var("CPLUS_INCLUDE_PATH") {
            info!("CPLUS_INCLUDE_PATH env var found : {}", cplus_incl_path);
            build_args.extend(cplus_incl_path.split(':').map(|s| String::from("-I") + s));
        }

        if let Ok(library_path) = std::env::var("LIBRARY_PATH") {
            info!("LIBRARY_PATH env var found : {}", library_path);
            build_args.extend(library_path.split(':').map(|s| String::from("-L") + s));
        }

        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for c-original");
        write(&self.main_file_path, &self.code).expect("Unable to write to file for c-original");
        let mut cmd = Command::new(&self.compiler);
        let cmd = cmd
            .arg(&self.main_file_path)
            .arg("-o")
            .arg(&self.bin_path)
            .arg("-v")
            .args(&build_args);

        info!(
            "full gcc command emitted:\n{}\n",
            format!("{:?}", cmd).replace('\"', "")
        );

        let output = cmd.output().expect("Unable to start process");

        //TODO if relevant, return the error number (parse it from stderr)
        if !output.status.success() {
            let error_message = String::from_utf8(output.stderr).unwrap();
            info!("Full GCC error message: {}", error_message);
            let mut relevant_error = String::new();

            let mut break_loop = false;
            for line in error_message.lines() {
                if break_loop {
                    relevant_error = relevant_error + "\n" + line;
                    return Err(SniprunError::CompilationError(relevant_error));
                }
                if line.contains("error") {
                    // info!("breaking at position {:?}", line.split_at(line.find("error").unwrap()).1);
                    relevant_error += line
                        .split_at(line.find("error").unwrap())
                        .1
                        .trim_start_matches("error: ")
                        .trim_end_matches("error:")
                        .trim_start_matches("error");
                    break_loop = true;
                }
            }

            Err(SniprunError::CompilationError(relevant_error))
        } else {
            let compiler_output = String::from_utf8(output.stdout).unwrap();
            info!("compiler output:\n{}\n", compiler_output);
            Ok(())
        }
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        let output = Command::new(&self.bin_path)
            .args(&self.get_data().cli_args)
            .output()
            .expect("Unable to start process");
        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else if C_original::error_truncate(&self.get_data()) == ErrTruncate::Short {
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
mod test_c_original {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial(c_original)]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("printf(\"1=1\\n\");");
        let mut interpreter = C_original::new(data);
        let res = interpreter.run_at_level(SupportLevel::Bloc);

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert_eq!(string_result, "1=1\n");
    }

    #[test]
    #[serial(c_original)]
    fn compilerror() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("int a = 1"); // missing ";"
        let mut interpreter = C_original::new(data);
        let res = interpreter.run_at_level(SupportLevel::Bloc);

        match res {
            Err(SniprunError::CompilationError(_)) => (),
            _ => panic!("Compilation should have failed"),
        };
    }
}
