use crate::interpreters::import::*;

// Be sure to read the CONTRIBUTING.md file :-)

#[derive(Clone)]
#[allow(non_camel_case_types)]
// For example, Rust_original is a good name for the first rust interpreter
pub struct Plantuml_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,

    language_work_dir: String,
    main_file_path: String,
    output_mode: String,
}

//necessary boilerplate, you don't need to implement that if you want a Bloc support level
//interpreter (the easiest && most common)
impl ReplLikeInterpreter for Plantuml_original {}

impl Interpreter for Plantuml_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Plantuml_original> {
        //create a subfolder in the cache folder
        let lwd = data.work_dir.clone() + "/plantuml_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&lwd)
            .expect("Could not create directory for example");

        let mfp = lwd.clone() + "/main.uml";
        Box::new(Plantuml_original {
            data,
            support_level,
            code: String::new(),
            language_work_dir: lwd,
            main_file_path: mfp,
            output_mode: String::new(),
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("PlantUML"), // in 1st position of vector, used for info only
            //':set ft?' in nvim to get the filetype of opened file
            String::from("puml"),
            String::from("uml"),
            String::from("pu"),
            String::from("iuml"),
            String::from("plantuml"),
        ]
    }

    fn get_name() -> String {
        // get your interpreter name
        String::from("Plantuml_original")
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

    fn default_for_filetype() -> bool {
        true
    }

    fn behave_repl_like_default() -> bool {
        false
    }

    fn has_repl_capability() -> bool {
        false
    }

    fn get_max_support_level() -> SupportLevel {
        //define the max level support of the interpreter (see readme for definitions)
        SupportLevel::Bloc
    }

    fn fetch_code(&mut self) -> Result<(), SniprunError> {
        if let Some(nvim_instance) = self.data.nvim_instance.clone() {
            let mut real_nvim_instance = nvim_instance.lock().unwrap();
            //note: you probably don't have to modify, or even understand this function

            //check if we not on boundary of block, if so replace self.code with whole bloc
            let line_n = self.data.range[0];
            if self
                .data
                .current_line
                .trim_start()
                .to_lowercase()
                .starts_with("@startuml")
            {
                let end_line = real_nvim_instance
                    .get_current_buf()
                    .unwrap()
                    .line_count(&mut real_nvim_instance)
                    .unwrap();
                let capped_end_line = std::cmp::min(line_n + 400, end_line); // in case there is a very long file, don't search for nothing up to line 500k
                let it = line_n + 1..capped_end_line + 1;

                let mut code_bloc = vec![];
                for i in it {
                    let line_i = real_nvim_instance
                        .get_current_buf()
                        .unwrap()
                        .get_lines(&mut real_nvim_instance, i - 1, i, false)
                        .unwrap()
                        .join("");
                    if line_i.trim_start().to_lowercase().starts_with("@enduml") {
                        //found end of bloc
                        info!("found end of bloc at line {}", i);
                        self.data.current_bloc = code_bloc.join("\n");
                    } else {
                        info!("adding line {} to current bloc", i);
                        code_bloc.push(line_i.to_string());
                    }
                }
            }
        }
        //
        //add code from data to self.code
        if !self
            .data
            .current_bloc
            .replace(&[' ', '\t', '\n', '\r'][..], "")
            .is_empty()
            && self.support_level >= SupportLevel::Bloc
        {
            // if bloc is not pseudo empty and has Bloc current support level,
            // add fetched code to self
            self.code.clone_from(&self.data.current_bloc);

        // if there is only data on current line / or Line is the max support level
        } else if !self.data.current_line.replace(' ', "").is_empty()
            && self.support_level >= SupportLevel::Line
        {
            self.code.clone_from(&self.data.current_line);
        } else {
            // no code was retrieved
            self.code = String::from("");
        }

        // now self.code contains the line or bloc of code wanted :-)
        Ok(())
    }

    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        if !self.code.contains("@startuml") {
            self.code = String::from("@startuml\n") + &self.code;
        }
        if !self.code.contains("@enduml") {
            self.code = self.code.clone() + "\n@enduml\n";
        }
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        let mut _file = File::create(&self.main_file_path)
            .expect("Failed to create file for plantuml_original");
        // IO errors can be ignored, or handled into a proper SniprunError
        // If you panic, it should not be too dangerous for anyone
        write(&self.main_file_path, &self.code)
            .expect("Unable to write to file for plantuml_original");

        self.output_mode = String::from("-tutxt");

        if let Some(config_value) =
            Plantuml_original::get_interpreter_option(&self.data, "output_mode")
        {
            if let Some(config_value_valid_string) = config_value.as_str() {
                self.output_mode = config_value_valid_string.to_string();
            }
        }
        let allowed_output_modes = [
            "-tutxt",
            "-thtml",
            "-tsvg",
            "-tpng",
            "-tpdf",
            "-teps",
            "-teps:text",
            "-tlatex",
            "-tlatex:nopreamble",
            "-ttxt",
        ];
        if !allowed_output_modes.contains(&self.output_mode.as_str()) {
            return Err(SniprunError::CustomError(format!(
                "invalid output mode {}, allowed modes are {:?}",
                self.output_mode, allowed_output_modes
            )));
        }

        let compiler = Plantuml_original::get_compiler_or(&self.data, "plantuml");
        //compile it (to the bin_path that already points to the rigth path)
        let output = Command::new(compiler.split_whitespace().next().unwrap())
            .args(compiler.split_whitespace().skip(1))
            .arg("-o")
            .arg(&self.language_work_dir)
            .arg(&self.output_mode)
            .arg("-nbthread")
            .arg("auto")
            .arg("-failfast2")
            .arg(self.main_file_path.clone())
            .output()
            .expect("Unable to start process");
        if output.status.success() {
            //return stdout
            Ok(())
        } else {
            // return stderr
            Err(SniprunError::CompilationError(
                String::from_utf8(output.stderr).unwrap(),
            ))
        }
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        let extension = String::from(".") + &self.output_mode[2..];
        let content = std::fs::read_to_string(self.main_file_path.replace(".uml", &extension));
        if let Ok(content) = content {
            Ok(content)
        } else {
            Err(SniprunError::RuntimeError(format!(
                "could not read output file {}",
                &self.main_file_path.replace(".uml", &extension)
            )))
        }
    }
}

// You can add tests if you want to
#[cfg(test)]
mod test_plantuml_original {
    use super::*;

    #[test]
    fn simple_print() {
        let mut data = DataHolder::new();

        //inspired from Rust syntax
        data.current_bloc = String::from("@startuml\nparticipant Bob");
        let mut interpreter = Plantuml_original::new(data);
        let res = interpreter.run();

        // -> should panic if not an Ok()
        let string_result = res.unwrap();

        // -> compare result with predicted
        assert!(string_result.contains("Bob"));
    }
}
