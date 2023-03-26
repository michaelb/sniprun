#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Mathematica_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,

    language_work_dir: String,
    main_file_path: String,

    current_output_id: u32,
}

impl Mathematica_original {
    fn remove_comment(&self, line: &str) -> String {
        let mut owned_line = line.to_owned();
        if let Some(index) = line.find("//") {
            owned_line.drain(index..);
        }
        owned_line
    }

    fn wrap_line_print_if_necessary(&self, line: &str) -> String {
        let line = self.remove_comment(line);
        if !line.contains("Print")
            && !line.contains("Plot")
            && !line.is_empty()
            && ![";", "[", "(", "{", "`"]
                .iter()
                .any(|&suffix| line.trim().ends_with(suffix))
            && !["\"", "}", "{", "[", "]", "(", ")"]
                .iter()
                .any(|&s| line.trim() == s)
        {
            return String::from("Print[") + &line + "];";
        }
        line
    }

    fn wait_out_file(&self, path: String, id: u32) -> Result<String, String> {
        let end_mark = String::from("\"sniprun_finished_id=") + &id.to_string() + "\"";
        let start_mark = String::from("\"sniprun_started_id=") + &id.to_string() + "\"";

        let error = "No valid password found";

        info!(
            "searching for things between {:?} and {:?}",
            start_mark, end_mark
        );

        let mut contents = String::new();

        let mut pause = std::time::Duration::from_millis(50);
        let _start = std::time::Instant::now();
        loop {
            std::thread::sleep(pause);
            pause = pause.saturating_add(std::time::Duration::from_millis(50));
            if let Ok(mut file) = std::fs::File::open(&path) {
                info!("file exists");
                let res = file.read_to_string(&mut contents);
                if res.is_ok() {
                    info!("file could be read : {:?}", contents);
                    // info!("file : {:?}", contents);
                    if contents.contains(&end_mark) {
                        info!("found");
                        break;
                    }
                    if contents.contains(error) {
                        return Err(
                            "No valid password found. Check :SnipInfo Mathematica_original"
                                .to_owned(),
                        );
                    }
                    contents.clear();
                }
            }
            info!("not found yet");

        }

        let index = contents.rfind(&start_mark).unwrap();
        Ok(
            contents[index + start_mark.len()..contents.len() - end_mark.len() - 1].to_owned(),
        )
    }
}

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
            current_output_id: 0,
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("Mathematica"),
            String::from("mma"),
            String::from("mathematica"),
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

    fn has_repl_capability() -> bool {
        true
    }

    fn default_for_filetype() -> bool {
        true
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
        let mut preload_graphics = String::from("");
        let mut wait_for_graphics = String::from("");
        if let Some(use_javagraphics_msgpack) =
            Mathematica_original::get_interpreter_option(&self.get_data(), "use_javagraphics_if_contains")
        {
            if let Some(use_javagraphics) = use_javagraphics_msgpack.as_array() {
                for test_contains_msgpack in use_javagraphics.iter() {
                    if let Some(test_contains) = test_contains_msgpack.as_str() {
                        if self.code.contains(test_contains) {
                            info!("Preloaded JavaGraphics");
                            preload_graphics = String::from("<<JavaGraphics`\n");
                            wait_for_graphics = String::from("Pause[3600];\n");

                            if let Some(time_mgspack) =
                                Mathematica_original::get_interpreter_option(&self.get_data(), "keep_plot_open_for")
                            {
                                if let Some(time) = time_mgspack.as_i64() {
                                    if time >= 0 {
                                        wait_for_graphics = "Pause[".to_owned() + &time.to_string() + "];";
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }

        if let Some(wrap_all_lines_with_print_msgpack) =
            Mathematica_original::get_interpreter_option(&self.get_data(), "wrap_all_lines_with_print")
        {
            if let Some(wrap_all_lines_with_print) = wrap_all_lines_with_print_msgpack.as_bool() {
                if wrap_all_lines_with_print {
                    let mut new_code = String::new();
                    for line in self.code.lines() {
                        new_code.push_str(&(self.wrap_line_print_if_necessary(line) + "\n"));
                    }
                    self.code = new_code;
                }
            }
        }
        if let Some(wrap_last_line_with_print_msgpack) =
            Mathematica_original::get_interpreter_option(&self.get_data(), "wrap_last_line_with_print")
        {
            if let Some(wrap_last_line_with_print) = wrap_last_line_with_print_msgpack.as_bool() {
                if wrap_last_line_with_print {
                    let mut new_code = self.code.lines().collect::<Vec<_>>();
                    let last_line = new_code.pop().unwrap_or("");
                    let last_line_modified = self.wrap_line_print_if_necessary(last_line) + "\n";
                    new_code.push(&last_line_modified);
                    self.code = new_code.join("\n");
                }
            }
        }

        info!("code: {}", self.code);
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
        let output = Command::new("WolframKernel")
            .arg("-noprompt")
            .arg("-script")
            .arg(&self.main_file_path)
            .output()
            .expect("Unable to start process");

        if output.status.success() {
            //return stdout
            Ok(String::from_utf8(output.stdout).unwrap())
        } else {
            // return stderr
            Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr).unwrap(),
            ))
        }
    }
}

impl ReplLikeInterpreter for Mathematica_original {
    fn fetch_code_repl(&mut self) -> Result<(), SniprunError> {
        self.fetch_code()?;

        if !self.read_previous_code().is_empty() {
            // nothing to do, kernel already running
            info!("Mathematica kernel already running");

            if let Some(id) = self.get_pid() {
                // there is a race condition here but honestly you'd have to
                // trigger it on purpose
                self.current_output_id = id + 1;
                self.set_pid(self.current_output_id);
            } else {
                info!("Could not retrieve a previous id even if the kernel is running");
            }

            Ok(())
        } else {
            // launch everything
            self.set_pid(0);

            let init_repl_cmd = self.data.sniprun_root_dir.clone()
                + "/src/interpreters/Mathematica_original/init_repl.sh";
            info!(
                "launching kernel : {:?} on {:?}",
                init_repl_cmd, &self.language_work_dir
            );

            match daemon() {
                Ok(Fork::Child) => {
                    let _res = Command::new("bash")
                        .args(&[init_repl_cmd, self.language_work_dir.clone()])
                        .output()
                        .unwrap();
                    let pause = std::time::Duration::from_millis(36_000_000);
                    std::thread::sleep(pause);

                    return Err(SniprunError::CustomError(
                        "Timeout expired for mathematica REPL".to_owned(),
                    ));
                }
                Ok(Fork::Parent(_)) => {}
                Err(_) => info!("Mathematica_original could not fork itself to the background"),
            };

            let pause = std::time::Duration::from_millis(100);
            std::thread::sleep(pause);
            self.save_code("kernel_launched".to_owned());

            Err(SniprunError::CustomError(
                "Mathematica kernel launched, re-run your snippet".to_owned(),
            ))
        }
    }
    fn add_boilerplate_repl(&mut self) -> Result<(), SniprunError> {
        info!("adding boilerplate");
        let mut preload_graphics = "";
        if let Some(use_javagraphics_msgpack) =
            Mathematica_original::get_interpreter_option(&self.get_data(), "use_javagraphics_if_contains")
        {
            if !self.read_previous_code().contains("JavaGraphics loaded") {
                if let Some(use_javagraphics) = use_javagraphics_msgpack.as_array() {
                    for test_contains_msgpack in use_javagraphics.iter() {
                        if let Some(test_contains) = test_contains_msgpack.as_str() {
                            if self.code.contains(test_contains) {
                                info!("Preloaded JavaGraphics");
                                self.save_code("JavaGraphics loaded".to_owned());
                                preload_graphics = "<<JavaGraphics`\n";
                                break;
                            }
                        }
                    }
                }
            } else {
                info!("not reloading JavaGraphics");
            }
        }

        let end_mark = String::from("\nPrint[\"sniprun_finished_id=")
            + &self.current_output_id.to_string()
            + "\"];\n";
        let start_mark = String::from("\nPrint[\"sniprun_started_id=")
            + &self.current_output_id.to_string()
            + "\"];\n";

        self.code = start_mark + preload_graphics + &self.code + &end_mark;

        info!("added boilerplate");
        Ok(())
    }
    fn build_repl(&mut self) -> Result<(), SniprunError> {
        self.build()?;
        Ok(())
    }
    fn execute_repl(&mut self) -> Result<String, SniprunError> {
        //run th binary and get the std output (or stderr)

        info!("running launcher");
        let send_repl_cmd = self.data.sniprun_root_dir.clone()
            + "/src/interpreters/Mathematica_original/launcher.sh";
        let res = Command::new(send_repl_cmd)
            .arg(self.language_work_dir.clone() + "/main.mma")
            .arg(self.language_work_dir.clone() + "/pipe_in")
            .spawn()
            .expect("could not run launcher");
        info!("launcher launched : {:?}", res);

        let outfile = self.language_work_dir.clone() + "/out_file";
        info!("outfile : {:?}", outfile);
        match self.wait_out_file(outfile, self.current_output_id) {
            Ok(s) => Ok(s),
            Err(s) => Err(SniprunError::CustomError(s)),
        }
    }
}
