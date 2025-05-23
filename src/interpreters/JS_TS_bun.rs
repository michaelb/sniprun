#![allow(clippy::zombie_processes)]
use crate::interpreters::import::*;

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct JS_TS_bun {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    cache_dir: String,

    current_output_id: u32,
    main_file_path: String,
}

impl JS_TS_bun {
    fn wait_out_file(&self, out_path: String, id: u32) -> Result<String, SniprunError> {
        let end_mark = String::from("sniprun_finished_id=") + &id.to_string();
        let start_mark = String::from("sniprun_started_id=") + &id.to_string();

        info!(
            "searching for things between {:?} and {:?}",
            start_mark, end_mark
        );

        let mut out_contents = String::new();

        let mut pause = std::time::Duration::from_millis(50);
        let start = std::time::Instant::now();
        loop {
            std::thread::sleep(pause);
            pause = pause.saturating_add(std::time::Duration::from_millis(50));

            // timeout after 30s if no result found
            if start.elapsed().as_secs() > JS_TS_bun::get_repl_timeout(&self.data) {
                return Err(SniprunError::InterpreterLimitationError(String::from(
                    "reached the repl timeout",
                )));
            }

            //check for stdout
            if let Ok(mut file) = std::fs::File::open(&out_path) {
                info!("file exists");
                out_contents.clear();
                let res = file.read_to_string(&mut out_contents);
                if res.is_ok() {
                    // info!("out {}", out_contents);
                    let relevant_content: String = strip_ansi_escapes::strip_str(&out_contents)
                        .lines()
                        .filter(|l| !l.contains("undefined"))
                        .filter(|l| !l.contains("> "))
                        .collect::<Vec<&str>>()
                        .join("\n");
                    info!("relevant {}", relevant_content);
                    // info!("file : {:?}", contents);
                    if relevant_content.contains(&end_mark) {
                        info!("out found");
                        let index = relevant_content.rfind(&start_mark).unwrap();
                        let index_end = relevant_content.rfind(&end_mark).unwrap();
                        // with bun not-prod-ready repl, it's posssible to have output after the markers
                        // I won't wait until I can be sure the error is finished printing
                        // but, I'll suppose it's <50 ms away
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        let possible_other =
                            if !relevant_content[index_end + end_mark.len()..].is_empty() {
                                relevant_content[index_end + end_mark.len()..].to_string()
                            } else {
                                String::new()
                            };
                        return Ok(relevant_content[index + start_mark.len()..index_end]
                            .to_owned()
                            + "\n"
                            + &possible_other);
                    }
                }
            }

            info!("not found yet");
        }
    }
}

impl Interpreter for JS_TS_bun {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<JS_TS_bun> {
        //create a subfolder in the cache folder
        let lwd = data.work_dir.clone() + "/js-ts_bun";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&lwd)
            .expect("Could not create directory for example");

        //pre-create string pointing to main file's and binary's path
        let mfp = lwd.clone() + "/main.ts";
        Box::new(JS_TS_bun {
            cache_dir: lwd + "/" + &JS_TS_bun::get_nvim_pid(&data),
            data,
            support_level,
            code: String::new(),
            main_file_path: mfp,
            current_output_id: 0,
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("TS/JS (bun)"), // in 1st position of vector, used for info only
            //':set ft?' in nvim to get the filetype of opened file
            String::from("typescript"),
            String::from("typescriptreact"),
            String::from("ts"), //should not be necessary, but just in case
            String::from("js"),
            String::from("javascript"),
        ]
    }

    fn get_name() -> String {
        // get your interpreter name
        String::from("JS_TS_bun")
    }

    fn get_current_level(&self) -> SupportLevel {
        self.support_level
    }
    fn set_current_level(&mut self, level: SupportLevel) {
        self.support_level = level;
    }

    fn default_for_filetype() -> bool {
        false
    }
    fn get_data(&self) -> DataHolder {
        self.data.clone()
    }

    fn get_max_support_level() -> SupportLevel {
        //define the max level support of the interpreter (see readme for definitions)
        SupportLevel::Bloc
    }

    fn behave_repl_like_default() -> bool {
        false
    }

    fn has_repl_capability() -> bool {
        true
    }

    fn fetch_code(&mut self) -> Result<(), SniprunError> {
        //note: you probably don't have to modify, or even understand this function

        //here if you detect conditions that make higher support level impossible,
        //or unecessary, you should set the current level down. Then you will be able to
        //ignore maybe-heavy code that won't be needed anyway

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
            // no code was retrieved
            self.code = String::from("");
        }

        // now self.code contains the line or bloc of code wanted :-)
        info!("javascript/typescript self.code) = {}", self.code);
        Ok(())
    }

    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        //write code to file
        let mut _file =
            File::create(&self.main_file_path).expect("failed to create file for js_ts_bun");
        // io errors can be ignored, or handled into a proper sniprunerror
        // if you panic, it should not be too dangerous for anyone
        write(&self.main_file_path, &self.code).expect("unable to write to file for js_ts_bun");

        Ok(())
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        //run the binary and get the std output (or stderr)
        //
        let mut bun_opts = String::new();
        if let Some(opts) = JS_TS_bun::get_interpreter_option(&self.data, "bun_run_opts") {
            if let Some(bun_opt_str) = opts.as_str() {
                bun_opts = bun_opt_str.to_string();
            }
        }
        let interpreter = JS_TS_bun::get_interpreter_or(&self.data, "bun");
        let output = Command::new(interpreter.split_whitespace().next().unwrap())
            .args(interpreter.split_whitespace().skip(1))
            .arg("run")
            .arg("--silent")
            .args(bun_opts.split_whitespace())
            .arg(&self.main_file_path)
            .output()
            .expect("Unable to start process");

        if output.status.success() {
            //return stdout
            Ok(String::from_utf8(output.stdout).unwrap())
        } else {
            // return stderr
            if JS_TS_bun::error_truncate(&self.get_data()) == ErrTruncate::Short {
                Err(SniprunError::RuntimeError(
                    String::from_utf8(output.stderr.clone())
                        .unwrap()
                        .lines()
                        .filter(|l| l.contains("Error:"))
                        .next_back()
                        .unwrap_or(&String::from_utf8(output.stderr).unwrap())
                        .to_string(),
                ))
            } else {
                Err(SniprunError::RuntimeError(
                    String::from_utf8(output.stderr).unwrap(),
                ))
            }
        }
    }
}

impl ReplLikeInterpreter for JS_TS_bun {
    fn fetch_code_repl(&mut self) -> Result<(), SniprunError> {
        if !self.read_previous_code().is_empty() {
            // nothing to do, kernel already running
            info!("bun kernel already running");

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
                            JS_TS_bun::get_nvim_pid(&self.data),
                            String::from("script --return --quiet -c 'bun repl' /dev/null"),
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
        self.add_boilerplate()?;
        let start_mark = String::from("\nconsole.log(\"sniprun_started_id=")
            + &self.current_output_id.to_string()
            + "\");\n";
        let end_mark = String::from("\nconsole.log(\"sniprun_finished_id=")
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
        self.code = start_mark + &all_code + &end_mark;
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
mod test_ts_js_bun_original {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial(bun)]
    fn simple_print() {
        let mut data = DataHolder::new();

        //inspired from Rust syntax
        data.current_bloc = String::from("let message: string = 'Hi';\nconsole.log(message);");
        let mut interpreter = JS_TS_bun::new(data);
        let res = interpreter.run_at_level(SupportLevel::Bloc);

        // -> should panic if not an Ok()
        let string_result = res.unwrap();

        // -> compare result with predicted
        assert_eq!(string_result, "Hi\n");
    }
    #[test]
    #[serial(bun)]
    fn print_repl() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("let message: string = 'Hi';\nconsole.log(message);");
        let mut interpreter = JS_TS_bun::new(data);
        let res = interpreter.run_at_level_repl(SupportLevel::Bloc);
        assert!(res.is_err());
    }
}
