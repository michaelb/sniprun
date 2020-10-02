//Interpreter:| Haskell_original    | haskell     |
//############|_____________________|_____________|________________<- delimiters to help formatting,
//###########| Interpretername      | language    | comment
// Keep (but modify the first line after the :) if you wish to have this interpreter listedvia SnipList
#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Haskell_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,

    ///specific to haskell
    haskell_work_dir: String,
    bin_path: String,
    main_file_path: String,
}

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
        let mfp = rwd.clone() + "/main.hs";
        let bp = String::from(&mfp[..mfp.len() - 3]); // remove extension so binary is named 'main'
        Box::new(Haskell_original {
            data,
            support_level,
            code: String::from(""),
            haskell_work_dir: rwd,
            bin_path: bp,
            main_file_path: mfp,
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![String::from("haskell"), String::from("hs")]
    }

    fn get_name() -> String {
        String::from("Haskell_original")
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
            .arg("-o")
            .arg(self.bin_path.clone())
            .arg(&self.main_file_path)
            .output()
            .expect("Unable to start process");

        //TODO if relevant, return the error number (parse it from stderr)
        if !output.status.success() {
            return Err(SniprunError::CompilationError("".to_string()));
        } else {
            return Ok(());
        }
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        //run th binary and get the std output (or stderr)
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
