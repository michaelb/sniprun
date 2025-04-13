#![allow(clippy::zombie_processes)]
use crate::interpreters::import::*;

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Rust_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,

    ///specific to rust
    compiler: String,
    rust_work_dir: String,
    bin_path: String,
    main_file_path: String,

    // for repl
    current_output_id: u32,
    cache_dir: String,
}

impl Rust_original {
    fn fetch_config(&mut self) {
        let default_compiler = String::from("rustc");
        self.compiler = default_compiler;
        if let Some(used_compiler) =
            Rust_original::get_interpreter_option(&self.get_data(), "compiler")
        {
            if let Some(compiler_string) = used_compiler.as_str() {
                info!("Using custom compiler: {}", compiler_string);
                self.compiler = compiler_string.to_string();
            }
        }
    }

    fn wait_out_file(
        &self,
        out_path: &str,
        err_path: &str,
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
            if start.elapsed().as_secs() > Rust_original::get_repl_timeout(&self.data) {
                return Err(SniprunError::InterpreterLimitationError(String::from(
                    "reached the repl timeout",
                )));
            }

            //check for stderr first
            if let Ok(mut file) = std::fs::File::open(err_path) {
                info!("file exists");
                err_contents.clear();
                let res = file.read_to_string(&mut err_contents);
                if res.is_ok() {
                    info!("file could be read : {:?}", err_contents);
                    // info!("file : {:?}", contents);
                    if err_contents.contains(&end_mark) {
                        info!("out found");
                        let index = err_contents.rfind(&start_mark).unwrap();
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

            //check for stdout
            if let Ok(mut file) = std::fs::File::open(out_path) {
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
}

impl Interpreter for Rust_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Rust_original> {
        //create a subfolder in the cache folder
        let rwd = data.work_dir.clone() + "/rust_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&rwd)
            .expect("Could not create directory for rust-original");

        //pre-create string pointing to main file's and binary's path
        let mfp = rwd.clone() + "/main.rs";
        let bp = String::from(&mfp[..mfp.len() - 3]); // remove extension so binary is named 'main'
        let cd = rwd.clone() + "/" + &Rust_original::get_nvim_pid(&data);
        Box::new(Rust_original {
            data,
            support_level,
            code: String::new(),
            rust_work_dir: rwd,
            bin_path: bp,
            main_file_path: mfp,
            compiler: String::new(),

            current_output_id: 0,
            cache_dir: cd,
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("Rust"),
            String::from("rust"),
            String::from("rust-lang"),
            String::from("rs"),
        ]
    }

    fn get_name() -> String {
        String::from("Rust_original")
    }

    fn has_repl_capability() -> bool {
        true
    }

    fn behave_repl_like_default() -> bool {
        false
    }

    fn default_for_filetype() -> bool {
        true
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
        Ok(())
    }

    fn fetch_code(&mut self) -> Result<(), SniprunError> {
        self.fetch_config();
        //add code from data to self.code
        if !self
            .data
            .current_bloc
            .replace(&[' ', '\t', '\n', '\r'][..], "")
            .is_empty()
            && self.support_level >= SupportLevel::Bloc
        {
            self.code.clone_from(&self.data.current_bloc);
        } else if !self.data.current_line.replace(' ', "").is_empty()
            && self.support_level >= SupportLevel::Line
        {
            self.code.clone_from(&self.data.current_line);
        } else {
            self.code = String::from("");
        }
        Ok(())
    }

    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        if !Rust_original::contains_main("fn main", &self.code, "//") {
            self.code = String::from("fn main() {") + &self.code + "\n}";
        }
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        //write code to file
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for rust-original");
        write(&self.main_file_path, &self.code).expect("Unable to write to file for rust-original");

        //compile it (to the bin_path that arleady points to the rigth path)
        let output = Command::new(self.compiler.split_whitespace().next().unwrap())
            .args(self.compiler.split_whitespace().skip(1))
            .arg("--out-dir")
            .arg(&self.rust_work_dir)
            .arg(&self.main_file_path)
            .output()
            .expect("Unable to start process");

        //TODO if relevant, return the error number (parse it from stderr)
        if !output.status.success() {
            let error_message = String::from_utf8(output.stderr).unwrap();
            //
            //take first line and remove first 'error' word (redondant)
            let first_line = error_message
                .lines()
                .next()
                .unwrap_or_default()
                .trim_start_matches("error: ")
                .trim_start_matches("error");
            Err(SniprunError::CompilationError(first_line.to_owned()))
        } else {
            Ok(())
        }
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        //run th binary and get the std output (or stderr)
        let output = Command::new(&self.bin_path)
            .args(&self.get_data().cli_args)
            .output()
            .expect("Unable to start process");
        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else if Rust_original::error_truncate(&self.get_data()) == ErrTruncate::Short {
            Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr.clone())
                    .unwrap()
                    .lines()
                    .next()
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

#[cfg(test)]
mod test_rust_original {
    use super::*;
    use crate::error::SniprunError;

    #[test]
    fn all_rust() {
        simple_print();
        runtime_error();
    }

    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("println!(\"HW, 1+1 = {}\", 1+1);");
        let mut interpreter = Rust_original::new(data);
        let res = interpreter.run();

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert_eq!(string_result, "HW, 1+1 = 2\n");
    }

    fn runtime_error() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from(
            "
            let mock_input = \"153.2\";
            let _ = mock_input.parse::<i32>().unwrap();
            



            ", // > 4 lines so the message doesn't  get shortened
        );
        let expected = String::from("ParseIntError { kind: InvalidDigit }");
        let mut interpreter = Rust_original::new(data);
        let res = interpreter.run();

        assert!(res.is_err());
        // should panic if not an Err()
        if let Err(e) = res {
            match e {
                SniprunError::RuntimeError(full_message) => {
                    assert!(full_message.contains(&expected))
                }
                _ => panic!(
                    "Not the right error message, wanted {:?} and got {:?} instead",
                    expected, e
                ),
            }
        }
    }
}

impl ReplLikeInterpreter for Rust_original {
    fn fetch_code_repl(&mut self) -> Result<(), SniprunError> {
        if !self.read_previous_code().is_empty() {
            // nothing to do, kernel already running
            info!("evcxr kernel already running");

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
                            Rust_original::get_nvim_pid(&self.data),
                            String::from("evcxr"),
                        ])
                        .output()
                        .unwrap();

                    return Err(SniprunError::CustomError("bun REPL exited".to_owned()));
                }
                Ok(Fork::Parent(_)) => {}
                Err(_) => {
                    info!("JS_TS_bun could not fork itself to the background to launch the kernel")
                }
            };

            let pause = std::time::Duration::from_millis(100);
            std::thread::sleep(pause);
            self.save_code("kernel_launched\n".to_owned());
            let pause = std::time::Duration::from_millis(100);
            std::thread::sleep(pause);

            let v = vec![(self.data.range[0] as usize, self.data.range[1] as usize)];
            Err(SniprunError::ReRunRanges(v))
        }
    }

    fn add_boilerplate_repl(&mut self) -> Result<(), SniprunError> {
        let start_mark = String::from("\nprintln!(\"sniprun_started_id=")
            + &self.current_output_id.to_string()
            + "\");\n";
        let end_mark = String::from("\nprintln!(\"sniprun_finished_id=")
            + &self.current_output_id.to_string()
            + "\");\n";

        let start_mark_err = String::from("\neprintln!(\"sniprun_started_id=")
            + &self.current_output_id.to_string()
            + "\");\n";
        let end_mark_err = String::from("\neprintln!(\"sniprun_finished_id=")
            + &self.current_output_id.to_string()
            + "\");\n";

        // Removing empty lines
        // self.code = self
        //     .code
        //     .lines()
        //     .filter(|l| !l.trim().is_empty())
        //     .collect::<Vec<&str>>()
        //     .join("\n");

        let all_code = String::from("\n") + &self.code + "\n\n";
        self.code = start_mark + &start_mark_err + &all_code + &end_mark + &end_mark_err;
        Ok(())
    }

    fn build_repl(&mut self) -> Result<(), SniprunError> {
        //write code to file
        let mut _file =
            File::create(&self.main_file_path).expect("failed to create file for rust_original");
        // io errors can be ignored, or handled into a proper sniprunerror
        // if you panic, it should not be too dangerous for anyone
        write(&self.main_file_path, &self.code).expect("unable to write to file for rust_original");

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
        self.wait_out_file(&outfile, &errfile, self.current_output_id)
    }
}
