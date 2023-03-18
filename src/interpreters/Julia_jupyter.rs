#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Julia_jupyter {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    kernel_file: String,
    main_file_path: String,
    launcher_path: String,
}

impl Interpreter for Julia_jupyter {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<Julia_jupyter> {
        //create a subfolder in the cache folder
        let pwd = data.work_dir.clone() + "/julia_jupyter";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&pwd)
            .expect("Could not create directory for julia-jupyter");

        //pre-create string pointing to main file's and binary's path
        let mfp = pwd.clone() + "/main.jl";
        let lp = pwd.clone() + "/main.sh";

        let kp = pwd + "/kernel_sniprun.json";
        Box::new(Julia_jupyter {
            data,
            support_level: level,
            code: String::new(),
            kernel_file: kp,
            main_file_path: mfp,
            launcher_path: lp,
        })
    }

    fn get_name() -> String {
        String::from("Julia_jupyter")
    }

    fn behave_repl_like_default() -> bool {
        true
    }

    fn has_repl_capability() -> bool {
        true
    }

    fn get_supported_languages() -> Vec<String> {
        vec![String::from("Julia"), String::from("julia")]
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
        self.code = String::from("\nprintln(\"\")\n")
            + &unindent(&format!("{}{}", "\n", self.code.as_str()));
        //add a print newline because the jupyter prompt interferers with fetching the result
        Ok(())
    }
    fn build(&mut self) -> Result<(), SniprunError> {
        // info!("python code:\n {}", self.code);
        write(&self.main_file_path, &self.code).expect("Unable to write to file for julia_jupyter");
        Ok(())
    }
    fn execute(&mut self) -> Result<String, SniprunError> {
        let output = Command::new("julia")
            .arg(&self.main_file_path)
            .output()
            .expect("Unable to start process");
        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else if Julia_jupyter::error_truncate(&self.get_data()) == ErrTruncate::Short {
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
impl ReplLikeInterpreter for Julia_jupyter {
    fn fetch_code_repl(&mut self) -> Result<(), SniprunError> {
        self.fetch_code()?;
        if !std::path::Path::new(&self.kernel_file).exists() {
            info!("no kernel file found");
            return Err(SniprunError::RuntimeError(
                "No running kernel found".to_string(),
            ));
        }
        Ok(())
    }
    fn add_boilerplate_repl(&mut self) -> Result<(), SniprunError> {
        info!("begins add boilerplate repl");
        self.code = String::from("\nprintln(\"\")\n")
            + &unindent(&format!("{}{}", "\n", self.code.as_str()));

        Ok(())
    }

    fn build_repl(&mut self) -> Result<(), SniprunError> {
        let actual_command = String::from("echo")
            + " "
            + &String::from("'include(\"")
            + &self.main_file_path
            + "\")"
            + "' "
            + "|"
            + " "
            + "jupyter-console"
            + " "
            + "--existing"
            + " "
            + &self.kernel_file.clone()
            + " "
            + "--kernel=julia-1.5"
            + " "
            + "--simple-prompt"
            + " "
            + "--no-confirm"
            + " "
            + "--ZMQTerminalInteractiveShell.banner=\"\"";

        write(&self.launcher_path, &actual_command)
            .expect("Unable to write file for julia_jupyter");
        info!("command written to launcher:\n{}\n", actual_command);
        write(&self.main_file_path, &self.code).expect("Unable to write to file for julia_jupyter");
        Ok(())
    }

    fn execute_repl(&mut self) -> Result<String, SniprunError> {
        info!("starting executing repl: bash {}", &self.launcher_path);
        let output = Command::new("bash")
            .arg(&self.launcher_path)
            .output()
            .expect("failed to run command");
        info!(
            "executed command!, stdout = {:?}, stder = {:?}",
            output.stdout, output.stderr
        );
        let result = String::from_utf8(output.stdout).unwrap();
        let mut cleaned_result: Vec<_> = result.lines().collect();
        info!("collected result");

        // first and last lines are the [In] x: prompts from jupyter-console
        cleaned_result.remove(cleaned_result.len() - 1);
        cleaned_result.remove(1);
        cleaned_result.remove(0);
        info!("result: {:?}", cleaned_result);

        info!("cleaned result: {:?}", cleaned_result);
        if String::from_utf8(output.stderr.clone()).unwrap().is_empty() {
            Ok(cleaned_result.join("\n") + "\n")
        } else {
            return Err(SniprunError::RuntimeError(
                String::from_utf8(strip_ansi_escapes::strip(output.stderr.clone()).unwrap())
                    .unwrap()
                    .lines()
                    .last()
                    .unwrap_or(
                        &String::from_utf8(strip_ansi_escapes::strip(output.stderr).unwrap())
                            .unwrap(),
                    )
                    .to_owned(),
            ));
        }
    }
}

#[cfg(test)]
mod test_julia {
    use super::*;

    #[test]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("print(\"lol\");");
        let mut interpreter = Julia_jupyter::new(data);
        let res = interpreter.run_at_level(SupportLevel::Bloc);

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert_eq!(string_result.trim(), "lol");
    }
}
