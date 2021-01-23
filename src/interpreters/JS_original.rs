//Interpreter:| JS_original         | javascript  |
//############|_____________________|_____________|________________<- delimiters to help formatting,
//############| Interpretername     | language    | comment
// Keep (but modify the first line after the :) if you wish to have this interpreter listed via SnipList
#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct JS_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    js_work_dir: String,
    main_file_path: String,
}
impl ReplLikeInterpreter for JS_original {}
impl Interpreter for JS_original {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<JS_original> {
        let bwd = data.work_dir.clone() + "/js-original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&bwd)
            .expect("Could not create directory for js-original");
        let mfp = bwd.clone() + "/main.js";
        Box::new(JS_original {
            data,
            support_level: level,
            code: String::from(""),
            js_work_dir: bwd,
            main_file_path: mfp,
        })
    }

    fn get_name() -> String {
        String::from("JS_original")
    }

    fn get_supported_languages() -> Vec<String> {
        vec![String::from("js"), String::from("javascript")]
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
            File::create(&self.main_file_path).expect("Failed to create file for js-original");

        write(&self.main_file_path, &self.code).expect("Unable to write to file for js-original");
        Ok(())
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        let output = Command::new("node")
            .arg(&self.main_file_path)
            .output()
            .expect("Unable to start process");
        info!("yay from js interpreter");
        if output.status.success() {
            return Ok(String::from_utf8(output.stdout).unwrap());
        } else {
            return Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr).unwrap(),
            ));
        }
    }
}
