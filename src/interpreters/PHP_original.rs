use crate::interpreters::import::*;

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct PHP_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    interpreter: String,
    main_file_path: String,
    cache_dir: String,
    current_output_id: u32,
}
impl PHP_original {
    fn wait_out_file(
        &self,
        out_path: String,
        err_path: String,
        id: u32,
    ) -> Result<String, SniprunError> {
        let end_mark = String::from("sniprun_finished_id=") + &id.to_string();
        let end_mark_nl = end_mark.clone() + "\n";
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
            if start.elapsed().as_secs() > PHP_original::get_repl_timeout(&self.data) {
                return Err(SniprunError::InterpreterLimitationError(String::from(
                    "reached the repl timeout",
                )));
            }

            //check for stderr first
            if let Ok(mut file) = std::fs::File::open(&err_path) {
                info!("errfile exists");
                err_contents.clear();
                if file.read_to_string(&mut err_contents).is_ok() {
                    // info!("errfile could be read : {:?}", err_contents);
                    if let Some(end_index) = err_contents.rfind(&end_mark_nl) {
                        if let Some(index) = err_contents.rfind(&start_mark) {
                            let err_to_display =
                                err_contents[index + start_mark.len()..end_index].to_owned();
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
                if file.read_to_string(&mut out_contents).is_ok() {
                    // info!("file could be read : {:?}", out_contents);
                    if out_contents.contains(&end_mark) {
                        info!("out found");
                        // NOTE: Because PHP writes the prompt to stdout, we filter it out here.
                        // Using the cli.pager ini setting we could circumvent this, but that would require
                        // a custom solution to launch the interpreter.
                        let lines = out_contents
                            .lines()
                            .skip_while(|l| l != &start_mark)
                            .skip(1)
                            .take_while(|l| l != &end_mark)
                            // remove the php prompt lines
                            .filter(|l| !l.starts_with("php > "))
                            .collect::<Vec<&str>>();
                        return Ok(lines.join("\n"));
                    }
                }
            }

            info!("not found yet");
        }
    }
    fn fetch_config(&mut self) {
        let mut interpreter: String = "php".to_owned();

        let data = self.get_data();
        if let Some(interpreter_val) = PHP_original::get_interpreter_option(&data, "interpreter") {
            if let Some(interpreter_string) = interpreter_val.as_str() {
                interpreter = interpreter_string.to_owned();
            }
        }
        self.interpreter = interpreter;
    }
}

impl Interpreter for PHP_original {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<PHP_original> {
        //create a subfolder in the cache folder
        let rwd = data.work_dir.clone() + "/php_original";

        DirBuilder::new()
            .recursive(true)
            .create(&rwd)
            .expect("Could not create directory for PHP_original");

        // Main file path
        let mfp = rwd.clone() + "/main.php";

        Box::new(PHP_original {
            support_level: level,
            code: String::from(""),
            main_file_path: mfp,
            cache_dir: rwd + "/" + &PHP_original::get_nvim_pid(&data),
            interpreter: String::from(""),
            current_output_id: 0,
            data,
        })
    }

    fn check_cli_args(&self) -> Result<(), SniprunError> {
        Ok(())
    }

    fn get_name() -> String {
        String::from("PHP_original")
    }

    fn behave_repl_like_default() -> bool {
        false
    }
    fn has_repl_capability() -> bool {
        true
    }

    fn default_for_filetype() -> bool {
        true
    }

    fn get_supported_languages() -> Vec<String> {
        vec![String::from("php"), String::from("PHP")]
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
        self.fetch_config();
        if self.get_current_level() >= SupportLevel::Bloc
            && self.data.current_bloc.chars().any(|c| !c.is_whitespace())
        {
            self.code.clone_from(&self.data.current_bloc);
        } else if self.data.current_line.chars().any(|c| !c.is_whitespace())
            && self.get_current_level() >= SupportLevel::Line
        {
            self.code.clone_from(&self.data.current_line);
        } else {
            self.code = String::from("");
        }

        Ok(())
    }
    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        // Add <?php tag if '<?' or '<?php' not present
        if !self.code.trim_start().starts_with("<?") {
            let mut new_code = String::from("<?php\n\n");
            new_code.push_str(&self.code);
            self.code = new_code;
        }
        Ok(())
    }
    fn build(&mut self) -> Result<(), SniprunError> {
        write(&self.main_file_path, &self.code).expect("Unable to write to file for PHP_original");
        Ok(())
    }
    fn execute(&mut self) -> Result<String, SniprunError> {
        info!(
            "Executing PHP_original with interpreter: {:?}",
            self.interpreter
        );
        let output = Command::new(&self.interpreter)
            .arg(&self.main_file_path)
            .args(&self.get_data().cli_args)
            .output()
            .expect("Unable to start process");
        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else if PHP_original::error_truncate(&self.get_data()) == ErrTruncate::Short {
            Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr.clone())
                    .unwrap()
                    .lines()
                    .last()
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
impl ReplLikeInterpreter for PHP_original {
    fn fetch_code_repl(&mut self) -> Result<(), SniprunError> {
        if !self.read_previous_code().is_empty() {
            // nothing to do, kernel already running
            info!("PHP kernel already running");

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
                        .arg(init_repl_cmd)
                        .arg(&self.cache_dir)
                        .arg(PHP_original::get_nvim_pid(&self.data))
                        .arg(&self.interpreter)
                        .arg("-a")
                        .output()
                        .unwrap();

                    return Err(SniprunError::CustomError("PHP REPL exited".to_owned()));
                }
                Ok(Fork::Parent(_)) => {}
                Err(_) => info!(
                    "PHP_original could not fork itself to the background to launch the kernel"
                ),
            };

            self.save_code("kernel_launched\n".to_owned());
            let pause = std::time::Duration::from_millis(100);
            std::thread::sleep(pause);
            let v = vec![(self.data.range[0] as usize, self.data.range[1] as usize)];
            Err(SniprunError::ReRunRanges(v))
        }
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
    fn add_boilerplate_repl(&mut self) -> Result<(), SniprunError> {
        let start_mark = String::from("\n;print(\"sniprun_started_id=")
            + &self.current_output_id.to_string()
            + "\\n\");\n";
        let end_mark = String::from("\n;print(\"sniprun_finished_id=")
            + &self.current_output_id.to_string()
            + "\\n\");\n";
        let start_mark_err = String::from("\n;fwrite(STDERR, \"sniprun_started_id=")
            + &self.current_output_id.to_string()
            + "\\n\");\n";
        let end_mark_err = String::from("\n;fwrite(STDERR, \"sniprun_finished_id=")
            + &self.current_output_id.to_string()
            + "\\n\");\n";

        let all_code = String::from("\n") + &self.code + "\n\n";
        self.code =
            String::from(start_mark) + &start_mark_err + &all_code + &end_mark + &end_mark_err;
        Ok(())
    }
}

#[cfg(test)]
mod test_php_original {
    use super::*;
    use crate::test_main::*;
    use crate::*;

    #[test]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("echo \"Hello World\n\";");
        let mut interpreter = PHP_original::new(data);
        let res = interpreter.run_at_level(SupportLevel::Bloc);

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert_eq!(string_result, "Hello World\n");
    }

    #[allow(dead_code)]
    fn test_repl() {
        let mut event_handler = fake_event();
        event_handler.fill_data(&fake_msgpack());
        event_handler.data.filetype = String::from("php");
        event_handler.data.current_bloc = String::from("$a = 5;\n$b = 6;");
        event_handler.data.repl_enabled = vec![String::from("PHP_original")];
        event_handler.data.sniprun_root_dir = String::from(".");
        //run the launcher (that selects, init and run an interpreter)
        let launcher = launcher::Launcher::new(event_handler.data.clone());
        let _result = launcher.select_and_run();

        event_handler.data.current_bloc = String::from("echo $a + $b;");
        let launcher = launcher::Launcher::new(event_handler.data.clone());
        let result = launcher.select_and_run();
        assert!(result.is_ok());
    }
}
