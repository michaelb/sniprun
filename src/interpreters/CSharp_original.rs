#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct CSharp_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,

    ///specific to csharp
    compiler: String,
    bin_path: String,
    main_file_path: String,
}

impl CSharp_original {
    fn fetch_config(&mut self) {
        let default_compiler = String::from("csc");
        self.compiler = default_compiler;
        if let Some(used_compiler) =
            CSharp_original::get_interpreter_option(&self.get_data(), "compiler")
        {
            if let Some(compiler_string) = used_compiler.as_str() {
                info!("Using custom compiler: {}", compiler_string);
                self.compiler = compiler_string.to_string();
            }
        }
    }
}

impl ReplLikeInterpreter for CSharp_original {}
impl Interpreter for CSharp_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<CSharp_original> {
        //create a subfolder in the cache folder
        let rwd = data.work_dir.clone() + "/csharp_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&rwd)
            .expect("Could not create directory for csharp-original");

        //pre-create string pointing to main file's and binary's path
        let mfp = rwd + "/main.cs";
        let bp = String::from(&mfp[..mfp.len() - 3]); // remove extension so binary is named 'main'
        Box::new(CSharp_original {
            data,
            support_level,
            code: String::new(),
            bin_path: bp,
            main_file_path: mfp,
            compiler: String::new(),
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("C#"),
            String::from("csharp"),
            String::from("cs"),
        ]
    }

    fn get_name() -> String {
        String::from("CSharp_original")
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
        SupportLevel::Bloc
    }

    fn check_cli_args(&self) -> Result<(), SniprunError> {
        // All cli arguments are sendable to Csharp
        Ok(())
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
        if !CSharp_original::contains_main("static void Main(string[] args)", &self.code, "//") {
            self.code =
                String::from("using System; class Hello { static void Main(string[] args) {\n ")
                    + &self.code
                    + "\n} }";
        }

        if !CSharp_original::contains_main("using System", &self.code, "//") {
            self.code = String::from("using System;\n") + &self.code;
        }
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        //write code to file
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for csharp-original");
        write(&self.main_file_path, &self.code)
            .expect("Unable to write to file for csharp-original");

        //compile it (to the bin_path that arleady points to the rigth path)
        let output = Command::new(&self.compiler)
            .arg(String::from("-out:") + &self.bin_path)
            .arg(&self.main_file_path)
            .output()
            .expect("Unable to start process");

        //TODO if relevant, return the error number (parse it from stderr)
        if !output.status.success() {
            let error_message = String::from_utf8(output.stderr).unwrap();
            //take first line and remove first 'error' word (redondant)
            let first_line = error_message
                .lines()
                .next()
                .unwrap_or_default()
                .trim_start_matches("error: ")
                .trim_start_matches("error");
            Err(SniprunError::CompilationError(first_line.to_owned()))
        } else {
            Ok(())
        }
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        //run th binary and get the std output (or stderr)
        let output = Command::new("mono")
            .arg(&self.bin_path)
            .args(&self.get_data().cli_args)
            .output()
            .expect("Unable to start process");
        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else if CSharp_original::error_truncate(&self.get_data()) == ErrTruncate::Short {
            return Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr.clone())
                    .unwrap()
                    .lines()
                    .next()
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
mod test_csharp_original {
    use super::*;

    #[test]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("Console.WriteLine(\"Hello World!\");");
        let mut interpreter = CSharp_original::new(data);
        let res = interpreter.run();

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert_eq!(string_result, "Hello World!\n");
    }
}
