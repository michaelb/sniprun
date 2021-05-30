// Be sure to read the CONTRIBUTING.md file :-)

#[derive(Clone)]
#[allow(non_camel_case_types)]
// For example, Rust_original is a good name for the first rust interpreter
pub struct Mathematica_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,

    language_work_dir: String,
    main_file_path: String,
}

//necessary boilerplate, you don't need to implement that if you want a Bloc support level
//interpreter (the easiest && most common)
impl ReplLikeInterpreter for Mathematica_original {}

impl Interpreter for Mathematica_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Mathematica_original> {
        //create a subfolder in the cache folder
        let lwd = data.work_dir.clone() + "/mathematica_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&lwd)
            .expect("Could not create directory for example");

        //pre-create string pointing to main file's and binary's path
        let mfp = lwd.clone() + "/main.mma";
        Box::new(Mathematica_original {
            data,
            support_level,
            code: String::new(),
            language_work_dir: lwd,
            main_file_path: mfp,
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("Mathematica"),
            String::from("mathematica"),
            String::from("mma"),
        ]
    }

    fn get_name() -> String {
        String::from("Mathematica_original")
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
        let mut preload_graphics = String::from("");
        let mut wait_for_graphics = String::from("");
        if let Some(use_javagraphics_msgpack) =
            self.get_interpreter_option("use_javagraphics_if_contains")
        {
            if let Some(use_javagraphics) = use_javagraphics_msgpack.as_array() {
                for test_contains_msgpack in use_javagraphics.iter() {
                    if let Some(test_contains) = test_contains_msgpack.as_str() {
                        if self.code.contains(test_contains) {
                            info!("Preloaded JavaGraphics");
                            preload_graphics = String::from("<<JavaGraphics`\n");
                            wait_for_graphics = String::from("Pause[3600];\n");

                            if let Some(time_mgspack) =
                                self.get_interpreter_option("keep_plot_open_for")
                            {
                                if let Some(time) = time_mgspack.as_i64() {
                                    if time >= 0 {
                                        wait_for_graphics =
                                            String::from("Pause[".to_owned() + &time.to_string() + "];");
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }

        self.code = preload_graphics + &self.code + &wait_for_graphics;
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        //write code to file
        let mut _file = File::create(&self.main_file_path)
            .expect("Failed to create file for mathematica_original");
        // IO errors can be ignored, or handled into a proper SniprunError
        // If you panic, it should not be too dangerous for anyone
        write(&self.main_file_path, &self.code)
            .expect("Unable to write to file for mathematica_original");

        Ok(())
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        //run th binary and get the std output (or stderr)
        let output = Command::new("wolfram")
            .arg("-noprompt")
            .arg("-script")
            .arg(&self.main_file_path)
            .output()
            .expect("Unable to start process");

        if output.status.success() {
            //return stdout
            return Ok(String::from_utf8(output.stdout).unwrap());
        } else {
            // return stderr
            return Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr).unwrap(),
            ));
        }
    }
}
