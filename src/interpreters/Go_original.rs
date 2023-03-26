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
        self.compiler = default_compiler;
        if let Some(used_compiler) =
            Go_original::get_interpreter_option(&self.get_data(), "compiler")
        {
            if let Some(compiler_string) = used_compiler.as_str() {
                info!("Using custom compiler: {}", compiler_string);
                self.compiler = compiler_string.to_string();
            }
        }
    }
    fn fetch_imports(&mut self) -> Result<(), SniprunError> {
        if self.support_level < SupportLevel::Import {
            // still need the fmt package in its most likely form at least
            self.code = String::from("import \"fmt\"\n") + &self.code;
            return Ok(());
        }

        let mut file_content = vec![];
        let mut errored = true;
        if let Some(real_nvim_instance) = self.data.nvim_instance.clone() {
            info!("got real nvim isntance");
            let mut rvi = real_nvim_instance.lock().unwrap();
            if let Ok(buffer) = rvi.get_current_buf() {
                info!("got buffer");
                if let Ok(buf_lines) = buffer.get_lines(&mut rvi, 0, -1, false) {
                    info!("got lines in buffer");
                    file_content = buf_lines;
                    errored = false;
                }
            }
        }
        if errored {
            return Err(SniprunError::FetchCodeError);
        }

        let all_imports = Go_original::parse_imports(file_content);
        let used_imports: Vec<(&str, &str)> = all_imports
            .iter()
            .map(|(a, p)| (a.as_str(), p.as_str()))
            .filter(|s| self.import_used(s.0))
            .collect();
        info!("used imports are {:?}", used_imports);

        if used_imports.is_empty() {
            return Ok(());
        }

        let mut import_code = String::from("import (\n");
        for import in used_imports.iter() {
            let (alias, path) = import;
            import_code.push_str(alias);
            import_code.push(' ');
            import_code.push_str(path);
            import_code.push('\n');
        }
        import_code.push_str(")\n");

        self.code = import_code + &self.code;

        Ok(())
    }
    fn import_used(&self, import: &str) -> bool {
        let r = Regex::new(&format!("[^a-zA-Z\\d_]{}\\.", import)).unwrap();
        r.is_match(&self.code)
    }

    fn parse_imports(s: Vec<String>) -> Vec<(String, String)> {
        // returns a list of "Name", "Path" for all imports
        let mut vec_imports = vec![];
        let mut in_import_bracket = false;
        for l in s {
            if l.trim().starts_with("import") {
                if l.contains('(') {
                    in_import_bracket = true;
                    continue;
                } else {
                    // lone import
                    let chunks: Vec<&str> = l.split_whitespace().skip(1).collect();
                    if let Some(alias_path) = Go_original::import_pathname(chunks) {
                        vec_imports.push(alias_path);
                    }
                }
            }
            if l.contains(')') && in_import_bracket {
                in_import_bracket = false;
            }

            if in_import_bracket {
                let chunks: Vec<&str> = l.split_whitespace().collect();
                if let Some(alias_path) = Go_original::import_pathname(chunks) {
                    vec_imports.push(alias_path);
                }
            }
        }
        vec_imports
    }

    fn import_pathname(vec: Vec<&str>) -> Option<(String, String)> {
        match vec.len() {
            0 => None,
            1 => Some((Go_original::parse_import_path(vec[0]), vec[0].to_string())),
            2 => Some((vec[0].to_string(), vec[1].to_string())),
            _ => None,
        }
    }

    fn parse_import_path(p: &str) -> String {
        p.replace('\"', "").split('/').last().unwrap().to_string()
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
        SupportLevel::Import
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
        if !Go_original::contains_main("func main (", &self.code, "//") {
            self.code = String::from("func main() {") + &self.code + "}";
        }

        if !Go_original::contains_main("import", &self.code, "//") {
            self.fetch_imports()?;
        } else {
            warn!("import keyword detected in code: Sniprun should fetch the needed imports by itself");
        }

        if Go_original::contains_main("package main", &self.code, "//") {
            warn!("\"package main\" detected in code: don't include that; sniprun adds it itself");
        }
        self.code = self.code.replace("package main", ""); //remove possibly and put it another at the right place
        self.code = String::from("package main\n") + &self.code;

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
            if Go_original::error_truncate(&self.get_data()) == ErrTruncate::Short {
                Err(SniprunError::CompilationError(
                    String::from_utf8(output.stderr.clone())
                        .unwrap()
                        .lines()
                        .last()
                        .unwrap_or(&String::from_utf8(output.stderr).unwrap())
                        .to_owned(),
                ))
            } else {
                Err(SniprunError::CompilationError(
                    String::from_utf8(output.stderr).unwrap(),
                ))
            }
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
        } else if Go_original::error_truncate(&self.get_data()) == ErrTruncate::Short {
            Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr.clone())
                    .unwrap()
                    .lines()
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

#[cfg(test)]
mod test_go_original {
    use super::*;

    use serial_test::serial;

    #[test]
    #[serial(go)]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("fmt.Println(\"Hello\")");
        let mut interpreter = Go_original::new_with_level(data, SupportLevel::Bloc);
        let res = interpreter.run();

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert_eq!(string_result, "Hello\n");
    }
}
