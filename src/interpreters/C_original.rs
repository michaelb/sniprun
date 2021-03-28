#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct C_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    c_work_dir: String,
    bin_path: String,
    main_file_path: String,
    compiler: String,
    imports: Vec<String>,
}

impl C_original {
    pub fn fetch_imports(&mut self) -> std::io::Result<()> {
        if self.support_level < SupportLevel::Import {
            return Ok(());
        }
        let mut file = File::open(&self.data.filepath)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        for line in contents.lines() {
            if line.starts_with("#include <") {
                self.imports.push(line.to_string());
            }
        }
        info!("fecthed imports : {:?}", self.imports);
        Ok(())
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
        let mfp = rwd.clone() + "/main.c";
        let bp = String::from(&mfp[..mfp.len() - 2]);
        Box::new(C_original {
            data,
            support_level,
            code: String::from(""),
            c_work_dir: rwd,
            bin_path: bp,
            main_file_path: mfp,
            compiler: String::from("gcc"),
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

    fn fetch_code(&mut self) -> Result<(), SniprunError> {
        if !self
            .data
            .current_bloc
            .replace(&[' ', '\t', '\n', '\r'][..], "")
            .is_empty()
        {
            self.code = self.data.current_bloc.clone();
        } else if !self.data.current_line.replace(" ", "").is_empty() {
            self.code = self.data.current_line.clone();
        } else {
            self.code = String::from("");
        }
        Ok(())
    }

    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        let res = self.fetch_imports();
        if res.is_err() {
            return Err(SniprunError::FetchCodeError);
        }
        self.code = String::from("int main() {\n") + &self.code + &"\nreturn 0;}";
        if !self.imports.iter().any(|s| s.contains("<stdio.h>")) {
            self.code = String::from("#include <stdio.h>\n") + &self.code;
        }
        self.code = self.imports.join("\n") + &"\n" + &self.code;

        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        info!("starting build");
        //write code to file
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for c-original");
        write(&self.main_file_path, &self.code).expect("Unable to write to file for c-original");
        let output = Command::new(&self.compiler)
            .arg(&self.main_file_path)
            .arg("-o")
            .arg(&self.bin_path)
            .output()
            .expect("Unable to start process");

        //TODO if relevant, return the error number (parse it from stderr)
        if !output.status.success() {
            let error_message = String::from_utf8(output.stderr).unwrap();
            info!("Returning nice C error message: {}", error_message);
            let mut relevant_error = String::new();

            let mut break_loop = false;
            for line in error_message.lines() {
                if break_loop {
                    relevant_error = relevant_error + "\n" + &line;
                    return Err(SniprunError::CompilationError(relevant_error));
                }
                if line.contains("error") {
                    // info!("breaking at position {:?}", line.split_at(line.find("error").unwrap()).1);
                    relevant_error = relevant_error
                        + line
                            .split_at(line.find("error").unwrap())
                            .1
                            .trim_start_matches("error: ")
                            .trim_end_matches("error:")
                            .trim_start_matches("error");
                    break_loop = true;
                }
            }

            return Err(SniprunError::CompilationError(relevant_error));
        } else {
            return Ok(());
        }
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        let output = Command::new(&self.bin_path)
            .output()
            .expect("Unable to start process");
        if output.status.success() {
            return Ok(String::from_utf8(output.stdout).unwrap());
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

    #[test]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("printf(\"1=1\\n\");");
        let mut interpreter = C_original::new(data);
        let res = interpreter.run_at_level(SupportLevel::Bloc);

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert_eq!(string_result, "1=1\n");
    }
}
