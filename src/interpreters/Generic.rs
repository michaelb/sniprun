#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Generic {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    glot_bin_path: String,
    main_file_path: String,
}
impl ReplLikeInterpreter for Generic {}
impl Interpreter for Generic {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Generic> {
        let rwd = data.work_dir.clone() + "/generic";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&rwd)
            .expect("Could not create directory for generic");
        let mfp = rwd + "/main.json";
        let bp = String::from(&data.sniprun_root_dir) + "/ressources/runner";
        Box::new(Generic {
            data,
            support_level,
            code: String::from(""),
            glot_bin_path: bp,
            main_file_path: mfp,
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![]
    }

    fn get_name() -> String {
        String::from("Generic")
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
        SupportLevel::Unsupported
        //actually this has no importance here
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
        self.code = String::from("{")
            + "\"language\":\""
            + &self.data.filetype
            + "\",\"files\":[{\"name\": \"name.any\",\"content\":\""
            + &self.code.replace("\\\"", "\"").replace("\"", "\\\"")
            + "\"}]}";
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        //write json file for glot
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for generic");
        write(&self.main_file_path, &self.code).expect("Unable to write to file for generic");
        Ok(())
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        info!(
            "executing generic: args are glotpath:{}, jsonpath:{}",
            &self.glot_bin_path, &self.main_file_path
        );
        let output = Command::new(&self.glot_bin_path)
            .stdin(File::open(&self.main_file_path).unwrap())
            .output()
            .unwrap_or_else(|_| panic!("Unable to start process, bin path : {}",
                &self.glot_bin_path));
        info!(
            "generic executed, status.success?:{}",
            output.status.success()
        );
        if output.status.success() {
            //unwrap the json output
            let js = String::from_utf8(output.stdout).unwrap();
            info!("json output: {:?}", js);
            let parsed: serde_json::Value = serde_json::from_str(&js).unwrap();
            let res_stdout = parsed.get("stdout").unwrap().to_string();
            let res_stderr = parsed.get("stderr").unwrap().to_string();

            if !res_stdout.is_empty() {
                info!("res_stdout :{}", res_stdout);
                Ok(String::from("Generic interpreter (!): ") + &res_stdout)
            } else if !res_stderr.is_empty() {
                Err(SniprunError::RuntimeError(
                    String::from("Generic interpreter (!): ") + &res_stderr,
                ))
            } else {
                Err(SniprunError::CompilationError(String::from(
                    "Generic interpreter (!): unknown compilation error",
                )))
            }
        } else {
            //this should not happen but anyway
            Err(SniprunError::RuntimeError(
                String::from("Generic interpreter (!): ")
                    + &String::from_utf8(output.stderr).unwrap(),
            ))
        }
    }
}

#[cfg(test)]
mod test_generic {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial(generic)]
    fn simple_print_python() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("print(3)");
        data.filetype = String::from("python");
        data.sniprun_root_dir = std::env::current_dir().unwrap().display().to_string();
        let mut interpreter = Generic::new(data);
        let res = interpreter.run();

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert!(string_result.contains('3'));
    }
}
