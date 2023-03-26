#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct OCaml_fifo {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    main_file_path: String,
    cache_dir: String,

    interpreter: String,
    interpreter_repl: String,
    current_output_id: u32,
}

impl OCaml_fifo {
    fn wait_out_file(
        &self,
        out_path: String,
        id: u32,
    ) -> Result<String, SniprunError> {
        //extra nils come from the stdout & stderr mark prints themselves
        let end_mark_ok = String::from("sniprun_finished_id=") + &id.to_string();
        let start_mark_ok = String::from("sniprun_started_id=") + &id.to_string();

        info!(
            "searching for things between {:?} and {:?}",
            start_mark_ok, end_mark_ok
        );

        let mut out_contents = String::new();

        let mut pause = std::time::Duration::from_millis(50);
        let start = std::time::Instant::now();
        loop {
            std::thread::sleep(pause);
            pause = pause.saturating_add(std::time::Duration::from_millis(50));

            // timeout after 30s if no result found
            if start.elapsed().as_secs() > 30 {
                return Err(SniprunError::InterpreterLimitationError(String::from(
                    "reached the 30s timeout",
                )));
            }

            // ocaml REPL never prints to stderr or so it seems

            //check for stdout (if output starts with "Error: ", return as erros instead
            if let Ok(mut file) = std::fs::File::open(&out_path) {
                info!("file exists");
                out_contents.clear();
                let res = file.read_to_string(&mut out_contents);
                if res.is_ok() {
                    info!("file could be read : {:?}", out_contents);
                    // info!("file : {:?}", contents);
                    out_contents = out_contents.replace("- : unit = ()\n","");
                    if out_contents.contains(&end_mark_ok) {
                        info!("out found");
                        let index = out_contents.rfind(&start_mark_ok).unwrap();
                        let output = out_contents[index + start_mark_ok.len()
                            ..out_contents.len() - end_mark_ok.len() - 1]
                            .to_owned();
                        if output.trim().contains("Error: ") {
                            return Err(SniprunError::RuntimeError(output));
                        } else {
                            return Ok(output);
                        }
                    }
                }
            }

            info!("not found yet");
        }
    }

    fn fetch_config(&mut self) {
        let default_interpreter_repl =
            String::from("ocaml -noprompt");
        let default_interpreter = String::from("ocaml");
        self.interpreter = default_interpreter;
        self.interpreter_repl = default_interpreter_repl;
        if let Some(used_interpreter) =
            OCaml_fifo::get_interpreter_option(&self.get_data(), "interpreter")
        {
            if let Some(interpreter_string) = used_interpreter.as_str() {
                info!("Using custom interpreter: {}", interpreter_string);
                self.interpreter = interpreter_string.to_string();
            }
        }
        if let Some(used_interpreter_repl) =
            OCaml_fifo::get_interpreter_option(&self.get_data(), "interpreter_repl")
        {
            if let Some(interpreter_string_repl) = used_interpreter_repl.as_str() {
                info!("Using custom interpreter: {}", interpreter_string_repl);
                self.interpreter_repl = interpreter_string_repl.to_string();
            }
        }
    }
}

impl Interpreter for OCaml_fifo {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<OCaml_fifo> {
        //create a subfolder in the cache folder
        let rwd = data.work_dir.clone() + "/ocaml_fifo";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&rwd)
            .expect("Could not create directory for ocaml-fifo");

        //pre-create string pointing to main file's and binary's path
        let mfp = rwd.clone() + "/main.ml";

        Box::new(OCaml_fifo {
            cache_dir: rwd + "/" + &OCaml_fifo::get_nvim_pid(&data),
            data,
            support_level: level,
            code: String::from(""),
            main_file_path: mfp,
            current_output_id: 0,
            interpreter: String::new(),
            interpreter_repl: String::new(),
        })
    }

    fn get_name() -> String {
        String::from("OCaml_fifo")
    }

    fn default_for_filetype() -> bool {
        true
    }

    fn behave_repl_like_default() -> bool {
        false
    }

    fn has_repl_capability() -> bool {
        true
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("OCaml"),
            String::from("ocaml"),
            String::from("ml"),
            String::from("mli"),
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
        SupportLevel::Bloc
    }

    fn fetch_code(&mut self) -> Result<(), SniprunError> {
        self.fetch_config();
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
        write(&self.main_file_path, &self.code).expect("Unable to write to file for ocaml_fifo");
        Ok(())
    }
    fn execute(&mut self) -> Result<String, SniprunError> {
        let output = Command::new(self.interpreter.clone())
            .arg(&self.main_file_path)
            .args(&self.get_data().cli_args)
            .output()
            .expect("Unable to start process");
        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else if OCaml_fifo::error_truncate(&self.get_data()) == ErrTruncate::Short {
            Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr)
                    .unwrap()
                    .lines()
                    .filter(|l| !l.to_lowercase().contains("error"))
                    .take(2)
                    .collect::<Vec<&str>>()
                    .join("\n"),
            ))
        } else {
            Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr).unwrap(),
            ))
        }
    }
}

impl ReplLikeInterpreter for OCaml_fifo {
    fn fetch_code_repl(&mut self) -> Result<(), SniprunError> {
        if !self.read_previous_code().is_empty() {
            // nothing to do, kernel already running
            info!("ocaml kernel already running");

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
                "launching kernel : {:?} on {:?} (interpreter is {}",
                init_repl_cmd, &self.cache_dir, self.interpreter_repl
            );

            match daemon() {
                Ok(Fork::Child) => {
                    let _res = Command::new("bash")
                        .args(&[
                            init_repl_cmd,
                            self.cache_dir.clone(),
                            Sage_fifo::get_nvim_pid(&self.data),
                            self.interpreter_repl.clone(),
                        ])
                        .output()
                        .unwrap();

                    return Err(SniprunError::CustomError("ocaml REPL exited".to_owned()));
                }
                Ok(Fork::Parent(_)) => {}
                Err(_) => info!(
                    "OCaml_fifo could not fork itself to the background to launch the kernel"
                ),
            };

            self.save_code("kernel_launched\n".to_owned());

            let pause = std::time::Duration::from_millis(300);
            std::thread::sleep(pause);

            Err(SniprunError::CustomError(
                "OCaml interactive kernel launched, re-run your snippet".to_owned(),
            ))
        }
    }

    fn add_boilerplate_repl(&mut self) -> Result<(), SniprunError> {
        self.add_boilerplate()?;
        let start_mark = String::from("\nprint_string \"sniprun_started_id=")
            + &self.current_output_id.to_string()
            + "\\n\" ;;\n";
        let end_mark = String::from("\nprint_string \"sniprun_finished_id=")
            + &self.current_output_id.to_string()
            + "\\n\" ;;\n";
        let start_mark_err = String::from("\nPrintf.eprintf  \"sniprun_started_id=")
            + &self.current_output_id.to_string()
            + "\\n\" ;;\n";
        let end_mark_err = String::from("\nPrintf.eprintf \"sniprun_finished_id=")
            + &self.current_output_id.to_string()
            + "\\n\" ;;\n";

        let all_code = String::from("\n") + &self.code + "\n\n";
        self.code = start_mark_err + &start_mark + &all_code + &end_mark_err + &end_mark;
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
        info!("outfile : {:?}", outfile);
        self.wait_out_file(outfile, self.current_output_id)
    }
}

#[cfg(test)]
mod test_ocaml_fifo {
    use super::*;

    #[test]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("print_string \"lol\\n\"");
        let mut interpreter = OCaml_fifo::new(data);
        let res = interpreter.run_at_level(SupportLevel::Bloc);
        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert_eq!(string_result, "lol\n");
    }
}
