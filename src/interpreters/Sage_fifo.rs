#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Sage_fifo {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    imports: String,
    main_file_path: String,
    cache_dir: String,

    interpreter: String,
    current_output_id: u32,
    user_sage_config: bool,
}

impl Sage_fifo {
    fn wait_out_file(
        &self,
        out_path: String,
        err_path: String,
        id: u32,
    ) -> Result<String, SniprunError> {
        let end_mark = String::from("sniprun_finished_id=") + &id.to_string() + "\n";
        let start_mark = String::from("sniprun_started_id=") + &id.to_string();

        info!(
            "searching for things between {:?} and {:?}",
            start_mark, end_mark
        );

        let mut out_contents = String::new();
        let mut err_contents = String::new();

        let mut pause = std::time::Duration::from_millis(50);
        let _start = std::time::Instant::now();
        loop {
            std::thread::sleep(pause);
            pause = pause.saturating_add(std::time::Duration::from_millis(50));

            //check for stderr first
            if let Ok(mut file) = std::fs::File::open(&err_path) {
                info!("errfile exists");
                out_contents.clear();
                let res = file.read_to_string(&mut err_contents);
                if res.is_ok() {
                    info!("errfile could be read : {:?}", err_contents);
                    // info!("file : {:?}", contents);
                    if err_contents.contains(&end_mark) {
                        if let Some(index) = err_contents.rfind(&start_mark) {
                            let err_to_display = err_contents
                                [index + start_mark.len()..err_contents.len() - end_mark.len() - 1]
                                .to_owned();
                            info!("err to display : {:?}", err_to_display);
                            if !err_to_display.trim().is_empty() {
                                info!("err found");
                                return Err(SniprunError::RuntimeError(err_to_display));
                            }
                        }
                    }
                }
            }

            //check for stdout
            if let Ok(mut file) = std::fs::File::open(&out_path) {
                info!("file exists");
                out_contents.clear();
                let res = file.read_to_string(&mut out_contents);
                if res.is_ok() {
                    info!("file could be read : {:?}", out_contents);
                    // info!("file : {:?}", contents);
                    out_contents = out_contents.replace("sage: ", "");
                    if out_contents.contains(&end_mark) {
                        info!("out found");
                        let index = out_contents.rfind(&start_mark).unwrap();
                        let out_contents_current = out_contents
                            [index + start_mark.len()..out_contents.len() - end_mark.len() - 1]
                            .to_string();

                        //check it's not actually an error
                        let error_indicators = [
                            "AssertionError",
                            "AttributeError",
                            "EOFError",
                            "FloatingPointError",
                            "GeneratorExit",
                            "ImportError",
                            "IndexError",
                            "KeyError",
                            "KeyboardInterrupt",
                            "MemoryError",
                            "NameError",
                            "NotImplementedError",
                            "OSError",
                            "OverflowError",
                            "ReferenceError",
                            "RuntimeError",
                            "StopIteration",
                            "SyntaxError",
                            "IndentationError",
                            "TabError",
                            "SystemError",
                            "ModuleNotFoundError",
                        ];
                        for try_error_indicator in error_indicators.iter() {
                            if out_contents_current.contains(try_error_indicator) {
                                info!("stdout contains error indicator");
                                err_contents = out_contents.clone();
                                // info!("file : {:?}", contents);
                                err_contents = err_contents.replace("sage: ", "");
                                err_contents = err_contents.replace("---------------------------------------------------------------------------\n","");
                                if err_contents.contains(&end_mark) {
                                    if let Some(index) = err_contents.rfind(&start_mark) {
                                        let err_to_display = err_contents[index + start_mark.len()
                                            ..err_contents.len() - end_mark.len() - 1]
                                            .to_owned();
                                        info!("err to display : {:?}", err_to_display);
                                        if !err_to_display.trim().is_empty() {
                                            info!("err found");
                                            return Err(SniprunError::RuntimeError(err_to_display));
                                        }
                                    }
                                }
                            }
                        }

                        return Ok(out_contents
                            [index + start_mark.len()..out_contents.len() - end_mark.len() - 1]
                            .to_owned());
                    }
                }
            }

            info!("not found yet");
        }
    }

    fn fetch_python_imports(&mut self) -> Result<(), SniprunError> {
        if self.support_level < SupportLevel::Import {
            return Ok(());
        }

        let mut v = vec![];
        let mut errored = true;
        if let Some(real_nvim_instance) = self.data.nvim_instance.clone() {
            info!("got real nvim instance");
            let mut rvi = real_nvim_instance.lock().unwrap();
            if let Ok(buffer) = rvi.get_current_buf() {
                info!("got buffer");
                if let Ok(buf_lines) = buffer.get_lines(&mut rvi, 0, -1, false) {
                    info!("got lines in buffer");
                    v = buf_lines;
                    errored = false;
                }
            }
        }
        if errored {
            return Err(SniprunError::FetchCodeError);
        }

        info!("lines are : {:?}", v);

        if !self
            .data
            .current_bloc
            .replace(&[' ', '\t', '\n', '\r'][..], "")
            .is_empty()
        {
            self.code = self.data.current_bloc.clone();
        }
        for line in v.iter() {
            // info!("lines are : {}", line);
            if line.contains("import ") //basic selection
                && !line.trim().starts_with('#')
            && self.module_used(line, &self.code)
            {
                // embed in try catch blocs in case uneeded module is unavailable

                let already_imported: String = self.read_previous_code();
                if !already_imported.contains(line) {
                    self.imports = self.imports.clone() + "\n" + line;
                    self.save_code(already_imported + "\n" + line);
                }
            }
        }
        info!("import founds : {:?}", self.imports);
        Ok(())
    }
    fn module_used(&self, line: &str, code: &str) -> bool {
        info!(
            "checking for python module usage: line {} in code {}",
            line, code
        );
        if line.contains('*') {
            return true;
        }
        if line.contains(" as ") {
            if let Some(name) = line.split(' ').last() {
                return code.contains(name);
            }
        }
        for name in line
            .replace(',', " ")
            .replace("from", " ")
            .replace("import ", " ")
            .split(' ')
            .filter(|&x| !x.is_empty())
        {
            if code.contains(name.trim()) {
                return true;
            }
        }
        false
    }

    fn fetch_config(&mut self) {
        let default_interpreter = String::from("sage");
        self.interpreter = default_interpreter;
        if let Some(used_interpreter) =
            Sage_fifo::get_interpreter_option(&self.get_data(), "interpreter")
        {
            if let Some(interpreter_string) = used_interpreter.as_str() {
                info!("Using custom interpreter: {}", interpreter_string);
                self.interpreter = interpreter_string.to_string();
            }
        }
        if let Some(user_sage_config) =
            Sage_fifo::get_interpreter_option(&self.get_data(), "interpreter")
        {
            if let Some(_user_sage_config_str) = user_sage_config.as_str() {
                info!("Using user sage config");
                self.user_sage_config = true;
            }
        }
    }
}

impl Interpreter for Sage_fifo {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<Sage_fifo> {
        //create a subfolder in the cache folder
        let rwd = data.work_dir.clone() + "/sage_fifo";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&rwd)
            .expect("Could not create directory for sage-fifo");

        //pre-create string pointing to main file's and binary's path
        let mfp = rwd.clone() + "/main.sage";

        Box::new(Sage_fifo {
            cache_dir: rwd + "/" + &Sage_fifo::get_nvim_pid(&data),
            data,
            support_level: level,
            code: String::from(""),
            imports: String::from(""),
            main_file_path: mfp,
            current_output_id: 0,
            interpreter: String::new(),
            user_sage_config: false,
        })
    }

    fn get_name() -> String {
        String::from("Sage_fifo")
    }

    fn behave_repl_like_default() -> bool {
        true
    }

    fn has_repl_capability() -> bool {
        true
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("SageMath"),
            String::from("sage"),
            String::from("sage.python"),
        ]
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
        SupportLevel::Import
    }

    fn default_for_filetype() -> bool {
        true
    }

    fn fetch_code(&mut self) -> Result<(), SniprunError> {
        self.fetch_config();
        self.fetch_python_imports()?;
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
        Ok(())
    }
    fn build(&mut self) -> Result<(), SniprunError> {
        write(&self.main_file_path, &self.code).expect("Unable to write to file for sage_fifo");
        Ok(())
    }
    fn execute(&mut self) -> Result<String, SniprunError> {
        Err(SniprunError::InterpreterLimitationError(
            "Sage_fifo only works in REPL mode, please enable it".to_owned(),
        ))
    }
}

impl ReplLikeInterpreter for Sage_fifo {
    fn fetch_code_repl(&mut self) -> Result<(), SniprunError> {
        if !self.read_previous_code().is_empty() {
            // nothing to do, kernel already running
            info!("Sage kernel already running");

            if let Some(id) = self.get_pid() {
                // there is a race condition here but honestly you'd have to
                // trigger it on purpose
                self.current_output_id = id + 1;
                self.set_pid(self.current_output_id);
            } else {
                info!("Could not retrieve a previous id even if the kernel is running");
                info!("This was in saved code: {}", self.read_previous_code());
                return Err(SniprunError::CustomError(
                    "Sniprun failed to connect to the running kernel, please SnipReset".to_string(),
                ));
            }

            self.fetch_code()?;
            Ok(())
        } else {
            self.fetch_config();
            // launch everything
            self.set_pid(0);

            let init_repl_cmd = self.data.sniprun_root_dir.clone() + "/ressources/init_repl.sh";
            info!(
                "launching kernel : {:?} on {:?}",
                init_repl_cmd, &self.cache_dir
            );
            match daemon() {
                Ok(Fork::Child) => {
                    let nodotstage_arg = if self.user_sage_config {
                        ""
                    } else {
                        "--nodotsage"
                    };

                    let _res = Command::new("bash")
                        .args(&[
                            init_repl_cmd,
                            self.cache_dir.clone(),
                            Sage_fifo::get_nvim_pid(&self.data),
                            self.interpreter.clone(),
                            nodotstage_arg.to_string(),
                        ])
                        .output()
                        .unwrap();

                    return Err(SniprunError::CustomError(
                        "sage REPL exited".to_owned(),
                    ));
                }
                Ok(Fork::Parent(_)) => {}
                Err(_) => {
                    info!("Sage_fifo could not fork itself to the background to launch the kernel")
                }
            };

            let pause = std::time::Duration::from_millis(100);
            std::thread::sleep(pause);
            self.save_code("kernel_launched\n".to_string());

            Err(SniprunError::CustomError(
                "Sage kernel launched, re-run your snippet".to_owned(),
            ))
        }
    }

    fn add_boilerplate_repl(&mut self) -> Result<(), SniprunError> {
        self.add_boilerplate()?;
        let start_mark = String::from("\n\nprint(\"sniprun_started_id=")
            + &self.current_output_id.to_string()
            + "\")\n\n";
        let end_mark = String::from("\n\nprint(\"sniprun_finished_id=")
            + &self.current_output_id.to_string()
            + "\")\n\n";
        let start_mark_err = String::from("\nprint(\"sniprun_started_id=")
            + &self.current_output_id.to_string()
            + "\", file=sys.stderr)\n";
        let end_mark_err = String::from("\nprint(\"sniprun_finished_id=")
            + &self.current_output_id.to_string()
            + "\", file=sys.stderr)\n";

        // remove empty lines interpreted as 'enter' by the sage interpreter
        self.code = self
            .code
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<&str>>()
            .join("\n");

        let all_code = self.imports.clone() + "\n" + &self.code;
        self.code = String::from("\nimport sys\n\n")
            + &start_mark
            + &start_mark_err
            + &all_code
            + &end_mark
            + &end_mark_err;
        Ok(())
    }

    fn build_repl(&mut self) -> Result<(), SniprunError> {
        self.build()
    }

    fn execute_repl(&mut self) -> Result<String, SniprunError> {
        let send_repl_cmd = self.data.sniprun_root_dir.clone() + "/ressources/launcher_repl.sh";
        info!("running launcher {}", send_repl_cmd);
        let res = Command::new(send_repl_cmd)
            .arg(self.main_file_path.clone())
            .arg(self.cache_dir.clone() + "/fifo_repl/pipe_in")
            .spawn();
        info!("cmd status: {:?}", res);
        res.expect("could not run launcher");
        // info!("launcher launched : {:?}", res);

        let outfile = self.cache_dir.clone() + "/fifo_repl/out_file";
        let errfile = self.cache_dir.clone() + "/fifo_repl/err_file";
        info!("outfile : {:?}", outfile);
        self.wait_out_file(outfile, errfile, self.current_output_id)
    }
}

#[cfg(test)]
mod test_sage_fifo {
    use super::*;

    use serial_test::serial;

    #[test]
    #[serial(sage_fifo)]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("print(\"lol\",1);");
        let mut interpreter = Sage_fifo::new(data);
        let _ = interpreter.run_at_level_repl(SupportLevel::Bloc);
        let res = interpreter.run_at_level_repl(SupportLevel::Bloc);
        assert!(res.is_err());
    }

    #[test]
    #[serial(sage_fifo)]
    fn print_quote() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("print(\"->\\\"\",1);");
        let mut interpreter = Sage_fifo::new(data);
        let _ = interpreter.run_at_level_repl(SupportLevel::Bloc);
        let res = interpreter.run_at_level_repl(SupportLevel::Bloc);
        assert!(res.is_err());
    }
}
