#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Haskell_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,

    bin_path: String,
    main_file_path: String,
}
impl ReplLikeInterpreter for Haskell_original {}
impl Interpreter for Haskell_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Haskell_original> {
        //create a subfolder in the cache folder
        let rwd = data.work_dir.clone() + "/haskell_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&rwd)
            .expect("Could not create directory for haskell-original");

        //pre-create string pointing to main file's and binary's path
        let mfp = rwd + "/main.hs";
        let bp = String::from(&mfp[..mfp.len() - 3]); // remove extension so binary is named 'main'
        Box::new(Haskell_original {
            data,
            support_level,
            code: String::from(""),
            bin_path: bp,
            main_file_path: mfp,
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("Haskell"),
            String::from("haskell"),
            String::from("hs"),
        ]
    }

    fn get_name() -> String {
        String::from("Haskell_original")
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
        SupportLevel::Line
    }

    fn check_cli_args(&self) -> Result<(), SniprunError> {
        // All cli arguments are sendable to python
        // Though they will be ignored in REPL mode
        Ok(())
    }

    fn fetch_code(&mut self) -> Result<(), SniprunError> {
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
        self.code = String::from("main = ") + &self.code;
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        //write code to file
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for haskell-original");
        write(&self.main_file_path, &self.code)
            .expect("Unable to write to file for haskell-original");

        //compile it (to the bin_path that arleady points to the rigth path)
        info!(
            "haskell interpreter : main & bin paths are {}, {}",
            &self.main_file_path, &self.bin_path
        );
        let output = Command::new("ghc")
            .arg("-dynamic")
            .arg("-o")
            .arg(self.bin_path.clone())
            .arg(&self.main_file_path)
            .output()
            .expect("Unable to start process");

        info!("code : {:?}", &self.code);
        //TODO if relevant, return the error number (parse it from stderr)
        if !output.status.success() {
            Err(SniprunError::CompilationError(
                String::from_utf8(output.stderr).unwrap(),
            ))
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
        } else if Haskell_original::error_truncate(&self.get_data()) == ErrTruncate::Short {
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
mod test_haskell_original {
    use super::*;

    #[test]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_line = String::from("putStrLn \"Hi\"");
        let mut interpreter = Haskell_original::new(data);
        let res = interpreter.run();

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert_eq!(string_result, "Hi\n");
    }
}
