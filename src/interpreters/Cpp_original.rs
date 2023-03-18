#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Cpp_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    bin_path: String,
    main_file_path: String,
    compiler: String,
    imports: Vec<String>, //using, namespaces, and includes
}

impl Cpp_original {
    pub fn fetch_imports(&mut self) -> Result<(), SniprunError> {
        if self.support_level < SupportLevel::Import {
            return Ok(());
        }
        let mut v = vec![];
        let mut errored = true;
        if let Some(real_nvim_instance) = self.data.nvim_instance.clone() {
            info!("got real nvim isntance");
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
            if (line.starts_with("namespace") && line.contains('='))
                || line.starts_with("using")
                || line.starts_with("#include <")
            {
                self.imports.push(line.to_string());
            }
        }
        Ok(())
    }

    fn fetch_config(&mut self) {
        let default_compiler = String::from("g++");
        self.compiler = default_compiler;
        if let Some(used_compiler) = Cpp_original::get_interpreter_option(&self.get_data(), "compiler") {
            if let Some(compiler_string) = used_compiler.as_str() {
                info!("Using custom compiler: {}", compiler_string);
                self.compiler = compiler_string.to_string();
            }
        }
    }

}

impl ReplLikeInterpreter for Cpp_original {}
impl Interpreter for Cpp_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Cpp_original> {
        let rwd = data.work_dir.clone() + "/c_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&rwd)
            .expect("Could not create directory for cpp-original");
        let mfp = rwd + "/main.cpp";
        let bp = String::from(&mfp[..mfp.len() - 2]);
        Box::new(Cpp_original {
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
        vec![
            String::from("C++"),
            String::from("cpp"),
            String::from("c++"),
        ]
    }

    fn get_name() -> String {
        String::from("Cpp_original")
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

        if !Cpp_original::contains_main("int main (", &self.code, "//") {
            self.code = String::from("int main() {\n") + &self.code + "\nreturn 0;}";
        }
        if !self.imports.iter().any(|s| s.contains("<iostream>")) {
            self.code = String::from("#include <iostream>\n") + &self.code;
        }
        self.code = self.imports.join("\n") + "\n" + &self.code;
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        //write code to file
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for rust-original");
        write(&self.main_file_path, &self.code).expect("Unable to write to file for rust-original");
        let output = Command::new(&self.compiler)
            .arg(&self.main_file_path)
            .arg("-o")
            .arg(&self.bin_path)
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
mod test_cpp_original {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial(cpp)]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("int a = 1;\nstd::cout << a << std::endl;");
        let mut interpreter = Cpp_original::new(data);
        let res = interpreter.run_at_level(SupportLevel::Bloc);

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert_eq!(string_result, "1\n");
    }

    #[test]
    #[serial(cpp)]
    fn compilerror() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("int a = 1;\nstd::cout << a << std::endl"); // missing ";"
        let mut interpreter = Cpp_original::new(data);
        let res = interpreter.run_at_level(SupportLevel::Bloc);

        match res {
            Err(SniprunError::CompilationError(_)) => (),
            _ => panic!("Compilation should have failed")
        };
    }



   }
