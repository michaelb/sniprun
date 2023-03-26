#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Python3_fifo {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    imports: String,
    main_file_path: String,
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
        let end_mark = String::from("sniprun_finished_id=") + &id.to_string() + "\n";
        let start_mark = String::from("sniprun_started_id=") + &id.to_string();

        info!(
            "searching for things between {:?} and {:?}",
            start_mark, end_mark
        );

        let mut out_contents = String::new();
        let mut err_contents = String::new();

        let mut pause = std::time::Duration::from_millis(50);
        let start = std::time::Instant::now();
        loop {
            std::thread::sleep(pause);
            pause = pause.saturating_add(std::time::Duration::from_millis(50));

            // timeout after 30s if no result found
            if start.elapsed().as_secs() > 30 {
                return Err(SniprunError::InterpreterLimitationError(String::from("reached the 30s timeout")));
            }

            // Python3_fifo-specific things to workaround nonblocking plot issues
            if start.elapsed().as_millis() > 150 {
                let sync_repl_cmd = self.data.sniprun_root_dir.clone() + "/ressources/sync_repl.sh";
                let res = Command::new(sync_repl_cmd).arg(self.cache_dir.clone()).output();
                info!(
                    "had to sync the repl because of timeout on awaiting result:\
                    happens  when a blocking command (plot, infinite loop) is run: {:?}",
                    res
                );
            }

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
                            let mut err_to_display = err_contents
                                [index + start_mark.len()..err_contents.len() - end_mark.len() - 1]
                                .to_owned();
                            info!("err to display : {:?}", err_to_display);
                            if !err_to_display.trim().is_empty() {
                                info!("err found");
                                if err_to_display.lines().count() > 2 {
                                    let mut err_to_display_vec =
                                        err_to_display.lines().skip(2).collect::<Vec<&str>>();
                                    err_to_display_vec.dedup();
                                    err_to_display = err_to_display_vec.join("\n");
                                }

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
                        return Ok(out_contents
                            [index + start_mark.len()..out_contents.len() - end_mark.len() - 1]
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
                if !already_imported.contains(line.trim()) {
                    let line = line.trim_start();
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

    /// needs imports to have been fetched already
    fn unblock_plot(&mut self) {
        let all_imports = self.imports.clone() + &self.read_previous_code();

        //it's not really pretty but should work most of the time
        if all_imports
            .split_whitespace()
            .collect::<String>()
            .contains("pyplotasplt")
        {
            self.code = self.code.replace("plt.show()", "plt.show(block=False)")
        }
        // self.code = self.code.replace("matplotlib.pyplot.show()", "matplotlib.pyplot.plause(0.001);matplotlib.pyplot.pause");
        self.code = self
            .code
            .replace("pyplot.show()", "pyplot.show(block=False)");
    }

    fn fetch_config(&mut self) {
        let default_interpreter = String::from("python3");
        self.interpreter = default_interpreter;
        if let Some(used_interpreter) =
            Python3_fifo::get_interpreter_option(&self.get_data(), "interpreter")
        {
            if let Some(interpreter_string) = used_interpreter.as_str() {
                info!("Using custom interpreter: {}", interpreter_string);
                self.interpreter = interpreter_string.to_string();
            }
        }

        if let Ok(path) = env::current_dir() {
            if let Some(venv_array_config) =
                Python3_fifo::get_interpreter_option(&self.get_data(), "venv")
            {
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

        Box::new(Python3_fifo {
            cache_dir: rwd + "/" + &Python3_fifo::get_nvim_pid(&data),
            data,
            support_level: level,
            code: String::from(""),
            imports: String::from(""),
            main_file_path: mfp,
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
        self.unblock_plot();

        if !self.imports.is_empty() {
            let mut indented_imports = String::new();
            for import in self.imports.lines() {
                indented_imports = indented_imports + "\t" + import + "\n";
            }

            self.imports = String::from("\ntry:\n") + &indented_imports + "\nexcept:\n\tpass\n";
        }

        let mut source_venv = String::new();
        if let Some(venv_path) = &self.venv {
            info!("loading venv: {}", venv_path);
            source_venv = source_venv + "\n" + "activate_this_file = \"" + venv_path + "\"";
            source_venv += "\nexec(compile(open(activate_this_file, \"rb\").read(), activate_this_file, 'exec'), dict(__file__=activate_this_file))\n";
        }

        self.code = source_venv
            + &self.imports.clone()
            + "\n## Imports above, code below, a #\\n# marker is very important to separate the try/catch bloc from the code  ##Here it is: #\n#"
            + &unindent(&format!("{}{}", "\n\n", self.code.as_str()));
        info!("source code::::: {}", self.code);
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
                    let _res = Command::new("bash")
                        .args(&[
                            init_repl_cmd,
                            self.cache_dir.clone(),
                            Python3_fifo::get_nvim_pid(&self.data),
                            self.interpreter.clone()
                                + " -ic 'import sys; sys.ps1=\"\";sys.ps2=\"\"'",
                        ])
                        .output()
                        .unwrap();

                    return Err(SniprunError::CustomError(
                        "python3 REPL exited".to_owned(),
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

        // remove empty lines interpreted as 'enter' by python
        self.code = self
            .code
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<&str>>()
            .join("\n")
            .replace("#\n#", "\n");

        // add empty lines (only containing correct indentation) to code when indentation decreases
        // unless it's a "except" or "finally" clause from a try-catch bloc
        let mut lines = vec![];
        for i in 0..(self.code.lines().count() - 1) {
            let l1 = self.code.lines().nth(i).unwrap();
            let l2 = self.code.lines().nth(i+1).unwrap();
            let nw1 = l1.len() - l1.trim_start().len();
            let nw2 = l2.len() - l2.trim_start().len();
            lines.push(l1);
            if nw1 > nw2 && !l2.trim().starts_with("except") && !l2.trim().starts_with("finally") {
                lines.push(&l2[0..nw2]);
            }
        }
        lines.push(self.code.lines().last().unwrap());

        self.code = lines.join("\n");


        let mut run_ion = String::new();
        let mut run_ioff = String::new();
        if self.imports.contains("pyplot") {
            run_ion.push_str(
                "try:\n\timport matplotlib.pyplot ;sniprun_ion_status_on = matplotlib.pyplot.ion()\nexcept:\n\tpass\n\n",
            );
            run_ioff.push_str("\nsniprun_ion_status_off = matplotlib.pyplot.ioff()\n");
        }

        let all_code = String::from("\n") + &self.code + "\n\n";
        self.code = String::from("\nimport sys\n\n")
            + &run_ion
            + &start_mark
            + &start_mark_err
            + &all_code
            + &end_mark
            + &end_mark_err
            + &run_ioff;
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

    #[test]
    fn module_usage() {
        let data = DataHolder::new();
        let interpreter = Python3_fifo::new(data);
        assert!(interpreter.module_used("import numpy as np", "print(np.array([1,2,3]))"));
        assert!(!interpreter.module_used("import numpy", "print(np.array([1,2,3]))"));
    }
}
