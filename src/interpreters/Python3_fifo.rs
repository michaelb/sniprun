#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Python3_fifo {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    imports: String,
    main_file_path: String,
    plugin_root: String,
    cache_dir: String,

    interpreter: String,
    venv: Option<String>,
    current_output_id: u32,
}

impl Python3_fifo {
    fn wait_out_file(
        &self,
        out_path: String,
        err_path: String,
        id: u32,
        ) -> Result<String, SniprunError> {
        let end_mark = String::from("sniprun_finished_id=") + &id.to_string()+"\n";
        let start_mark = String::from("sniprun_started_id=") + &id.to_string();

        info!(
            "searching for things between {:?} and {:?}",
            start_mark, end_mark
            );

        let mut out_contents = String::new();
        let mut err_contents = String::new();

        loop {
            let pause = std::time::Duration::from_millis(50);
            std::thread::sleep(pause);

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

            //check for stdout
            if let Ok(mut file) = std::fs::File::open(&out_path) {
                info!("file exists");
                out_contents.clear();
                let res = file.read_to_string(&mut out_contents);
                if res.is_ok() {
                    info!("file could be read : {:?}", out_contents);
                    // info!("file : {:?}", contents);
                    if out_contents.contains(&end_mark) {
                        info!("out found");
                        let index = out_contents.rfind(&start_mark).unwrap();
                        return Ok(out_contents[index + start_mark.len()
                                  ..out_contents.len() - end_mark.len() - 1]
                                  .to_owned());
                    }
                }
            }

            info!("not found yet");
        }
    }

    fn fetch_imports(&mut self) -> Result<(), SniprunError> {
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
            if (line.trim().starts_with("import ") || line.trim().starts_with("from"))  //basic selection
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
            .replace(",", " ")
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
        let default_interpreter = String::from("python3");
        if let Some(used_interpreter) = self.get_interpreter_option("interpreter") {
            if let Some(interpreter_string) = used_interpreter.as_str() {
                info!("Using custom interpreter: {}", interpreter_string);
                self.interpreter = interpreter_string.to_string();
            }
        }
        self.interpreter = default_interpreter;

        if let Ok(path) = env::current_dir() {
            if let Some(venv_array_config) = self.get_interpreter_option("venv") {
                if let Some(actual_vec_of_venv) = venv_array_config.as_array() {
                    for possible_venv in actual_vec_of_venv.iter() {
                        if let Some(possible_venv_str) = possible_venv.as_str() {
                            let venv_abs_path = path.to_str().unwrap().to_owned()
                                + "/"
                                + possible_venv_str
                                + "/bin/activate_this.py";
                            if std::path::Path::new(&venv_abs_path).exists() {
                                self.venv = Some(venv_abs_path);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Interpreter for Python3_fifo {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<Python3_fifo> {
        //create a subfolder in the cache folder
        let rwd = data.work_dir.clone() + "/python3_fifo";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&rwd)
            .expect("Could not create directory for python3-fifo");

        //pre-create string pointing to main file's and binary's path
        let mfp = rwd.clone() + "/main.py";

        let pgr = data.sniprun_root_dir.clone();
        Box::new(Python3_fifo {
            data,
            support_level: level,
            code: String::from(""),
            imports: String::from(""),
            main_file_path: mfp,
            plugin_root: pgr,
            cache_dir: rwd,
            current_output_id: 0,
            interpreter: String::new(),
            venv: None,
        })
    }

    fn get_name() -> String {
        String::from("Python3_fifo")
    }

    fn default_for_filetype() -> bool {
        false
    }

    fn behave_repl_like_default() -> bool {
        true
    }

    fn has_repl_capability() -> bool {
        true
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("Python 3"),
            String::from("python3"),
            String::from("python"),
            String::from("py"),
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

    fn fetch_code(&mut self) -> Result<(), SniprunError> {
        self.fetch_config();
        self.fetch_imports()?;
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
        write(&self.main_file_path, &self.code).expect("Unable to write to file for python3_fifo");
        Ok(())
    }
    fn execute(&mut self) -> Result<String, SniprunError> {
        Err(SniprunError::InterpreterLimitationError(
                "Python3_fifo only works in REPL mode, please enable it".to_owned(),
                ))
    }
}

impl ReplLikeInterpreter for Python3_fifo {
    fn fetch_code_repl(&mut self) -> Result<(), SniprunError> {

        if !self.read_previous_code().is_empty() {
            // nothing to do, kernel already running
            info!("Python3 kernel already running");

            if let Some(id) = self.get_pid() {
                // there is a race condition here but honestly you'd have to
                // trigger it on purpose
                self.current_output_id = id + 1;
                self.set_pid(self.current_output_id);
            } else {
                info!("Could not retrieve a previous id even if the kernel is running");
                info!("This was in saved code: {}", self.read_previous_code());
                return Err(SniprunError::CustomError("Sniprun failed to connect to the running kernel, please SnipReset".to_string()));
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
                    let _res = Command::new("bash")
                        .args(&[
                              init_repl_cmd,
                              self.cache_dir.clone(),
                              self.interpreter.clone()
                              + " -ic 'import sys; sys.ps1=\"\";sys.ps2=\"\"'",
                        ])
                        .output()
                        .unwrap();
                    let pause = std::time::Duration::from_millis(36_000_000);
                    std::thread::sleep(pause);

                    return Err(SniprunError::CustomError(
                            "Timeout expired for python3 REPL".to_owned(),
                            ));
                }
                Ok(Fork::Parent(_)) => {}
                Err(_) => info!(
                    "Python3_fifo could not fork itself to the background to launch the kernel"
                    ),
            };

            let pause = std::time::Duration::from_millis(100);
            std::thread::sleep(pause);
            self.save_code("kernel_launched\nimport sys".to_owned());

            Err(SniprunError::CustomError(
                    "Python3 kernel launched, re-run your snippet".to_owned(),
                    ))
        }

    }

    fn add_boilerplate_repl(&mut self) -> Result<(), SniprunError> {
        self.add_boilerplate()?;
        let start_mark = String::from("\nprint(\"sniprun_started_id=")
            + &self.current_output_id.to_string()
            + "\")\n";
        let end_mark = String::from("\nprint(\"sniprun_finished_id=")
            + &self.current_output_id.to_string()
            + "\")\n";
        let start_mark_err = String::from("\nprint(\"sniprun_started_id=")
            + &self.current_output_id.to_string()
            + "\", file=sys.stderr)\n";
        let end_mark_err = String::from("\nprint(\"sniprun_finished_id=")
            + &self.current_output_id.to_string()
            + "\", file=sys.stderr)\n";

        let all_code = self.imports.clone() + "\n" + &self.code;
        self.code = start_mark + &start_mark_err + &all_code + &end_mark + &end_mark_err;
        Ok(())
    }

    fn build_repl(&mut self) -> Result<(), SniprunError> {
        self.build()
    }

    fn execute_repl(&mut self) -> Result<String, SniprunError> {
        let send_repl_cmd = self.data.sniprun_root_dir.clone() + "/ressources/launcher_repl.sh";
        info!("running launcher {}", send_repl_cmd);
        let res = Command::new(send_repl_cmd)
            .arg(self.cache_dir.clone() + "/main.py")
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
mod test_python3_fifo {
    use super::*;

    use serial_test::serial;

    #[test]
    #[serial(pythonfifo)]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("print(\"lol\",1);");
        let mut interpreter = Python3_fifo::new(data);
        let res = interpreter.run_at_level_repl(SupportLevel::Bloc);
        assert!(res.is_err());
    }

    #[test]
    #[serial(pythonfifo)]
    fn print_quote() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("print(\"->\\\"\",1);");
        let mut interpreter = Python3_fifo::new(data);
        let res = interpreter.run_at_level_repl(SupportLevel::Bloc);
        assert!(res.is_err());
    }
}
