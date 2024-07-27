use crate::interpreters::import::*;

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Julia_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    main_file_path: String,
    cache_dir: String,

    interpreter: String,
    interpreter_args: Vec<String>, // for now, used for --project=....

    current_output_id: u32,
}

impl Julia_original {
    fn wait_out_file(&self, path: String, id: u32) -> Result<String, String> {
        let end_mark = String::from("sniprun_finished_id=") + &id.to_string();
        let start_mark = String::from("sniprun_started_id=") + &id.to_string();

        info!(
            "searching for things between {:?} and {:?}",
            start_mark, end_mark
        );

        let mut contents = String::new();

        let mut pause = std::time::Duration::from_millis(50);
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
                    contents.clear();
                }
            }
            info!("not found yet");
        }

        let index = contents.rfind(&start_mark).unwrap();
        Ok(contents[index + start_mark.len()..contents.len() - end_mark.len() - 1].to_owned())
    }

    fn fetch_config(&mut self) {
        let default_interpreter = String::from("julia");
        self.interpreter = default_interpreter;
        if let Some(used_interpreter) =
            Julia_original::get_interpreter_option(&self.get_data(), "interpreter")
        {
            if let Some(interpreter_string) = used_interpreter.as_str() {
                info!("Using custom interpreter: {}", interpreter_string);
                self.interpreter = interpreter_string.to_string();
            }
        }

        if let Some(project_opt) =
            Julia_original::get_interpreter_option(&self.get_data(), "project")
        {
            if let Some(mut project_str) = project_opt.as_str() {
                if project_str == "." {
                    info!(
                        "Using neovim's current working directory as julia --project: {}",
                        self.data.projectroot
                    );
                    project_str = &self.data.projectroot;
                }

                self.interpreter_args = vec![String::from("--project=") + project_str];
            }
        }
    }
}

impl Interpreter for Julia_original {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<Julia_original> {
        //create a subfolder in the cache folder
        let rwd = data.work_dir.clone() + "/julia_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&rwd)
            .expect("Could not create directory for julia-original");

        //pre-create string pointing to main file's and binary's path
        let mfp = rwd.clone() + "/main.jl";

        Box::new(Julia_original {
            data,
            support_level: level,
            code: String::from(""),
            main_file_path: mfp,
            interpreter: String::new(),
            interpreter_args: Vec::new(),
            cache_dir: rwd,
            current_output_id: 0,
        })
    }

    fn get_name() -> String {
        String::from("Julia_original")
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
            String::from("Julia"),
            String::from("julia"),
            String::from("jl"),
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

    fn check_cli_args(&self) -> Result<(), SniprunError> {
        // All cli arguments are sendable to julia
        // Though they will be ignored in REPL mode
        Ok(())
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
            self.code.clone_from(&self.data.current_bloc);
        } else if !self.data.current_line.replace(' ', "").is_empty()
            && self.get_current_level() >= SupportLevel::Line
        {
            self.code.clone_from(&self.data.current_line);
        } else {
            self.code = String::from("");
        }

        Ok(())
    }
    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        Ok(())
    }
    fn build(&mut self) -> Result<(), SniprunError> {
        // info!("python code:\n {}", self.code);
        write(&self.main_file_path, &self.code)
            .expect("Unable to write to file for julia_original");
        Ok(())
    }
    fn execute(&mut self) -> Result<String, SniprunError> {
        let output = Command::new(self.interpreter.split_whitespace().next().unwrap())
            .args(self.interpreter.split_whitespace().skip(1))
            .args(&self.interpreter_args)
            .arg(&self.main_file_path)
            .args(&self.get_data().cli_args)
            .output()
            .expect("Unable to start process");
        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else if Julia_original::error_truncate(&self.get_data()) == ErrTruncate::Short {
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

impl ReplLikeInterpreter for Julia_original {
    fn fetch_code_repl(&mut self) -> Result<(), SniprunError> {
        self.fetch_code()?;

        if !self.read_previous_code().is_empty() {
            // nothing to do, kernel already running
            info!("Julia kernel already running");

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
                + "/src/interpreters/Julia_original/init_repl.sh";
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
                            Julia_original::get_nvim_pid(&self.data),
                            self.interpreter.clone(),
                        ])
                        .args(&self.interpreter_args)
                        .output()
                        .unwrap();

                    return Err(SniprunError::CustomError("julia REPL exited".to_owned()));
                }
                Ok(Fork::Parent(_)) => {}
                Err(_) => info!(
                    "Julia_original could not fork itself to the background to launch the kernel"
                ),
            };

            let pause = std::time::Duration::from_millis(100);
            std::thread::sleep(pause);
            self.save_code("kernel_launched".to_owned());

            Err(SniprunError::CustomError(
                "Julia kernel launched, re-run your snippet".to_owned(),
            ))
        }
    }

    fn add_boilerplate_repl(&mut self) -> Result<(), SniprunError> {
        self.add_boilerplate()?;
        let start_mark = String::from("println(\"sniprun_started_id=")
            + &self.current_output_id.to_string()
            + "\")\n";
        let end_mark = String::from("println(\"sniprun_finished_id=")
            + &self.current_output_id.to_string()
            + "\")\n";

        let start_mark_err = String::from("println(stderr, \"sniprun_started_id=")
            + &self.current_output_id.to_string()
            + "\")\n";
        let end_mark_err = String::from("println(stderr, \"sniprun_finished_id=")
            + &self.current_output_id.to_string()
            + "\")\n";
        self.code =
            start_mark + &start_mark_err + "\n" + &self.code + "\n" + &end_mark_err + &end_mark;
        Ok(())
    }

    fn build_repl(&mut self) -> Result<(), SniprunError> {
        self.build()
    }

    fn execute_repl(&mut self) -> Result<String, SniprunError> {
        info!("running launcher");
        let send_repl_cmd = self.data.sniprun_root_dir.clone() + "/ressources/launcher_repl.sh";
        let res = Command::new(send_repl_cmd)
            .arg(self.main_file_path.clone())
            .arg(self.cache_dir.clone() + "/fifo_repl/pipe_in")
            .spawn()
            .expect("could not run launcher");
        info!("launcher launched : {:?}", res);

        let outfile = self.cache_dir.clone() + "/fifo_repl/out_file";
        info!("outfile : {:?}", outfile);
        match self.wait_out_file(outfile, self.current_output_id) {
            Ok(s) => Ok(s),
            Err(s) => Err(SniprunError::CustomError(s)),
        }
    }
}

#[cfg(test)]
mod test_julia_original {
    use super::*;

    #[test]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("println(\"hello\")");
        let mut interpreter = Julia_original::new(data);
        let res = interpreter.run();

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert_eq!(string_result, "hello\n");
    }
}
