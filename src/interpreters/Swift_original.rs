#![allow(clippy::zombie_processes)]
use crate::interpreters::import::*;

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Swift_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    imports: String,
    main_file_path: String,
    bin_path: String,
    cache_dir: String,

    compiler: String,
    interpreter: String,
    current_output_id: u32,
}

impl Swift_original {
    fn wait_out_file(
        &self,
        out_path: String,
        err_path: String,
        id: u32,
    ) -> Result<String, SniprunError> {
        let end_mark = String::from("sniprun_finished_id=") + &id.to_string();
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
            if start.elapsed().as_secs() > Swift_original::get_repl_timeout(&self.data) {
                return Err(SniprunError::InterpreterLimitationError(String::from(
                    "reached the repl timeout",
                )));
            }

            //check for stderr first
            let err_file_empty = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&err_path);
            info!("err_file_empty, {err_file_empty:?}");
            if let Ok(mut file) = err_file_empty {
                info!("errfile exists");
                err_contents.clear();
                let res = file.read_to_string(&mut err_contents);
                info!("errfile read staus :{:?}", res);
                if res.is_ok() {
                    info!("errfile could be read : {:?}", err_contents);
                    // info!("file : {:?}", contents);
                    let mut err_to_display = err_contents[..].to_owned();
                    info!("err to display : {:?}", err_to_display);
                    if !err_to_display.trim().is_empty() {
                        info!("err found");
                        if err_to_display.lines().count() > 0 {
                            let mut err_to_display_vec =
                                err_to_display.lines().collect::<Vec<&str>>();
                            err_to_display_vec.dedup();
                            err_to_display = err_to_display_vec.join("\n");
                        }

                        let _ = file.set_len(0); // rm old error for future code
                                                 // but since actually the repl will continue to write at the previous
                                                 // file cursor position, there will be many \0 bytes at the beginning
                        let err_to_display = err_to_display.replace("\0", "");
                        return Err(SniprunError::RuntimeError(err_to_display));
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
                            [index + start_mark.len()..out_contents.len() - end_mark.len() - 2]
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
            self.code.clone_from(&self.data.current_bloc);
        }
        for line in v {
            // info!("lines are : {}", line);
            if line.replace("@_exported", "").trim().starts_with("import ") {
                self.imports = self.imports.clone() + "\n" + &line;
            }
        }
        info!("import founds : {:?}", self.imports);
        Ok(())
    }

    fn fetch_config(&mut self) {
        let default_interpreter = String::from("swift repl");
        self.interpreter = default_interpreter;
        if let Some(used_interpreter) =
            Swift_original::get_interpreter_option(&self.get_data(), "interpreter")
        {
            if let Some(interpreter_string) = used_interpreter.as_str() {
                info!("Using custom interpreter: {}", interpreter_string);
                self.interpreter = interpreter_string.to_string();
            }
        }

        let default_compiler = String::from("swiftc");
        self.compiler = default_compiler;
        if let Some(used_compiler) =
            Swift_original::get_interpreter_option(&self.get_data(), "compiler")
        {
            if let Some(compiler_string) = used_compiler.as_str() {
                info!("using custom compiler: {}", compiler_string);
                self.compiler = compiler_string.to_string();
            }
        }
    }
}

impl Interpreter for Swift_original {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<Swift_original> {
        //create a subfolder in the cache folder
        let rwd = data.work_dir.clone() + "/swift_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&rwd)
            .expect("Could not create directory for swift-original");

        //pre-create string pointing to main file's and binary's path
        let mfp = rwd.clone() + "/main.swift";
        let bfp = rwd.clone() + "/main";

        Box::new(Swift_original {
            cache_dir: rwd + "/" + &Swift_original::get_nvim_pid(&data),
            data,
            support_level: level,
            code: String::from(""),
            imports: String::from(""),
            main_file_path: mfp,
            bin_path: bfp,
            current_output_id: 0,
            interpreter: String::new(),
            compiler: String::new(),
        })
    }

    fn get_name() -> String {
        String::from("Swift_original")
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
        vec![String::from("Swift"), String::from("swift")]
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
        self.code = self.imports.clone() + "\n" + &self.code;
        Ok(())
    }
    fn build(&mut self) -> Result<(), SniprunError> {
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for swift-original");
        write(&self.main_file_path, &self.code)
            .expect("Unable to write to file for swift-original");
        let mut cmd = Command::new(self.compiler.split_whitespace().next().unwrap());
        let cmd = cmd
            .args(self.compiler.split_whitespace().skip(1))
            .arg(&self.main_file_path)
            .arg("-o")
            .arg(&self.bin_path);

        info!(
            "full {} command emitted:\n{}\n",
            self.compiler,
            format!("{:?}", cmd).replace('\"', "")
        );

        let output = cmd.output().expect("Unable to start process");
        //TODO if relevant, return the error number (parse it from stderr)
        if !output.status.success() {
            let error_message = String::from_utf8(output.stderr).unwrap();
            Err(SniprunError::CompilationError(error_message))
        } else {
            let compiler_output = String::from_utf8(output.stdout).unwrap();
            info!("compiler output:\n{}\n", compiler_output);
            Ok(())
        }
    }
    fn execute(&mut self) -> Result<String, SniprunError> {
        let mut cmd = Command::new(&self.bin_path);
        cmd.args(&self.get_data().cli_args);

        info!("cmd: {:?}", &cmd);
        let output = cmd.output().expect("Unable to start process");
        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else if Swift_original::error_truncate(&self.get_data()) == ErrTruncate::Short {
            return Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr.clone())
                    .unwrap()
                    .lines()
                    .last()
                    .unwrap_or(&String::from_utf8(output.stderr).unwrap())
                    .to_owned(),
            ));
        } else {
            return Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr).unwrap(),
            ));
        }
    }
}

impl ReplLikeInterpreter for Swift_original {
    fn fetch_code_repl(&mut self) -> Result<(), SniprunError> {
        if !self.read_previous_code().is_empty() {
            // nothing to do, kernel already running
            info!("swift kernel already running");

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
            let mut cmd = Command::new("bash");
            cmd.args(&[
                init_repl_cmd.clone(),
                self.cache_dir.clone(),
                Swift_original::get_nvim_pid(&self.data),
                self.interpreter
                    .split_whitespace()
                    .next()
                    .unwrap()
                    .to_string(),
            ])
            .args(self.interpreter.split_whitespace().skip(1));
            info!("init repl cmd = {:?}", cmd);
            match daemon() {
                Ok(Fork::Child) => {
                    let mut cmd = Command::new("bash");
                    cmd.args(&[
                        init_repl_cmd,
                        self.cache_dir.clone(),
                        Swift_original::get_nvim_pid(&self.data),
                        self.interpreter
                            .split_whitespace()
                            .next()
                            .unwrap()
                            .to_string(),
                    ])
                    .args(self.interpreter.split_whitespace().skip(1));
                    info!("init repl cmd = {:?}", cmd);
                    let _res = cmd.output().unwrap();

                    return Err(SniprunError::CustomError("swift REPL exited".to_owned()));
                }
                Ok(Fork::Parent(_)) => {}
                Err(_) => info!(
                    "Swift_original could not fork itself to the background to launch the kernel"
                ),
            };

            self.save_code("kernel_launched\n".to_owned());
            let pause = std::time::Duration::from_millis(100);
            std::thread::sleep(pause);

            let v = vec![(self.data.range[0] as usize, self.data.range[1] as usize)];
            Err(SniprunError::ReRunRanges(v))
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
        // let start_mark_err = String::from("\nprint(\"sniprun_started_id=")
        //     + &self.current_output_id.to_string()
        //     + "\", file=sys.stderr)\n";
        // let end_mark_err = String::from("\nprint(\"sniprun_finished_id=")
        //     + &self.current_output_id.to_string()
        //     + "\", file=sys.stderr)\n";

        self.code = start_mark + &self.code + &end_mark;
        Ok(())
    }

    fn build_repl(&mut self) -> Result<(), SniprunError> {
        write(&self.main_file_path, &self.code).expect("Unable to write to file for python3_fifo");
        Ok(())
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
