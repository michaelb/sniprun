//Interpreter:| Ruby_original       | Ruby        |
//############|_____________________|_____________|________________<- delimiters to help formatting,
//###########| Interpretername      | language    | comment
// Keep (but modify the first line after the :) if you wish to have this interpreter listedvia SnipList
#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Ruby_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    ruby_work_dir: String,
    main_file_path: String,
}

impl Interpreter for Ruby_original {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<Ruby_original> {
        let bwd = data.work_dir.clone() + "/ruby-original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&bwd)
            .expect("Could not create directory for ruby-original");
        let mfp = bwd.clone() + "/main.rb";
        Box::new(Ruby_original {
            data,
            support_level: level,
            code: String::from(""),
            ruby_work_dir: bwd,
            main_file_path: mfp,
        })
    }

    fn get_name() -> String {
        String::from("Ruby_original")
    }

    fn get_supported_languages() -> Vec<String> {
        vec![String::from("ruby"), String::from("rb")]
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
        } else if !self.data.current_line.replace(" ", "").is_empty()
            && self.get_current_level() >= SupportLevel::Line
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
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for ruby-original");

        write(&self.main_file_path, &self.code).expect("Unable to write to file for ruby-original");
        Ok(())
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        let output = Command::new("ruby")
            .arg(&self.main_file_path)
            .output()
            .expect("Unable to start process");
        info!("yay from ruby interpreter");
        if output.status.success() {
            return Ok(String::from_utf8(output.stdout).unwrap());
        } else {
            return Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr).unwrap(),
            ));
        }
    }
}
