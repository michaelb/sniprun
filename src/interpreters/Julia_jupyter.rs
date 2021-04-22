#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Julia_jupyter {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    kernel_file: String,
    plugin_root: String,
    cache_dir: String,
}

impl Julia_jupyter {
    fn wait_on_kernel(&self) -> Result<(), SniprunError> {
        let step = std::time::Duration::from_millis(100);
        let mut timeout = std::time::Duration::from_millis(18000);
        loop {
            if let Ok(content) = std::fs::read_to_string(&self.kernel_file) {
                if !content.is_empty() {
                    info!("kernel file not empty, contains: {}", content);
                    return Ok(());
                }
            }
            std::thread::sleep(step);
            if let Some(remaining) = timeout.checked_sub(step) {
                timeout = remaining;
            } else {
                info!("Timeout on jupyter kernel start expired");
                return Err(SniprunError::CustomError(String::from(
                    "Timeout on jupyter kernel start expired",
                )));
            }
        }
    }

    fn init_kernel(&mut self) -> Result<(), SniprunError> {
        let mut saved_code = self.read_previous_code();
        if saved_code.is_empty() {
            //initialize kernel. Relying on self.read_previous_code to
            //know when to start a new kernel is important as
            //this will be cleared by the SnipReplMemoryClean command
            let _res = std::fs::remove_file(&self.kernel_file);
            let _res = Command::new("jupyter-kernel")
                .arg("--kernel=julia-1.5")
                .arg(String::from("--KernelManager.connection_file=") + &self.kernel_file)
                .spawn();

            self.wait_on_kernel()?;
            info!("Initialized kernel at {}", self.kernel_file);

            // Spawn an IOPub watcher
            let file_res = File::open(&self.kernel_file);
            if file_res.is_err() {
                info!("Failed to open kernel file");
            }
            let file = file_res.unwrap();
            let client_res = Client::from_reader(file);
            if client_res.is_err() {
                info!("client_res is not ok");
                return Err(SniprunError::CustomError(String::from(
                    "Error while trying to connect to the jupyter kernel",
                )));
            }
            let client = client_res.unwrap();
            let receiver_res = client.iopub_subscribe();
            if receiver_res.is_err() {
                info!("receiver_res is not ok");
                return Err(SniprunError::CustomError(String::from(
                    "Error while trying to connect to the jupyter kernel",
                )));
            }
            // let receiver = receiver_res.unwrap();
            // std::thread::spawn(move || {
            //     info!("Listener thread initialized");
            //     for msg in receiver {
            //         info!("Received message from kernel: {:#?}", msg);
            //     }
            // });
            // Set up the heartbeat watcher
            // let hb_receiver = client.heartbeat().unwrap();
            // std::thread::spawn(move || {
            //     for _ in hb_receiver {
            //         info!("Received heartbeat from kernel");
            //     }
            // });
            saved_code = self.kernel_file.to_string();
        } else {
            // kernel already running
            info!(
                "Using already loaded jupyter kernel at {}",
                self.kernel_file
            );
        }
        // save kernel
        self.save_code(saved_code);
        Ok(())
    }
}

impl Interpreter for Julia_jupyter {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<Julia_jupyter> {
        //create a subfolder in the cache folder
        let pwd = data.work_dir.clone() + "/julia_jupyter";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&pwd)
            .expect("Could not create directory for julia-jupyter");

        let pgr = data.sniprun_root_dir.clone();

        let kp = pwd.clone() + "/kernel_sniprun.json";
        Box::new(Julia_jupyter {
            data,
            support_level: level,
            code: String::new(),
            kernel_file: kp,
            plugin_root: pgr,
            cache_dir: pwd,
        })
    }

    fn get_name() -> String {
        String::from("Julia_jupyter")
    }

    fn behave_repl_like_default() -> bool {
        true
    }

    fn has_repl_capability() -> bool {
        true
    }

    fn get_supported_languages() -> Vec<String> {
        vec![String::from("Julia"), String::from("julia")]
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
        Ok(())
    }
    fn execute(&mut self) -> Result<String, SniprunError> {
        Err(SniprunError::CustomError(
            "Please enable REPL mode for the Julia_jupyter interpreter".to_string(),
        ))
    }
}
impl ReplLikeInterpreter for Julia_jupyter {
    fn fetch_code_repl(&mut self) -> Result<(), SniprunError> {
        self.fetch_code()?;
        self.init_kernel()?;
        Ok(())
    }
    fn add_boilerplate_repl(&mut self) -> Result<(), SniprunError> {
        info!("begins add boilerplate repl");
        self.code = unindent(&format!("{}{}", "\n", self.code.as_str()));
        Ok(())
    }

    fn build_repl(&mut self) -> Result<(), SniprunError> {
        Ok(())
    }

    fn execute_repl(&mut self) -> Result<String, SniprunError> {
        self.wait_on_kernel()?;

        let file = File::open(&self.kernel_file).unwrap();
        let client_res = Client::from_reader(file);
        if client_res.is_err() {
            info!("client_res is not ok");
            return Err(SniprunError::CustomError(String::from(
                "Error while trying to connect to the jupyter kernel",
            )));
        }

        let client = client_res.unwrap();


        // Command to run
        info!("running command");
        let command = jupyter_client::commands::Command::Execute {
            code: "a=1".to_string(),
            silent: false,
            store_history: false,
            user_expressions: HashMap::new(),
            allow_stdin: false,
            stop_on_error: false,
        };
        info!("command to send : {:?}", command);

        // Run some code on the kernel
        // std::thread::sleep_ms(1000);
        let response_res = client.send_shell_command(command);

        // std::thread::sleep_ms(1000);
        info!("command sent");
        // std::thread::sleep_ms(1000);
        if response_res.is_err() {
            info!("response_res is err");
            return Err(SniprunError::InternalError("could not send fetched code to the kernel".to_owned()));
        }
        let _response = response_res.unwrap();



        let mut cleaned_result = vec![String::new()];

        // first and last lines are the [In] x: prompts from jupyter-console
        cleaned_result.remove(cleaned_result.len() - 1);
        cleaned_result.remove(1);
        cleaned_result.remove(0);
        info!("result: {:?}", cleaned_result);

        let success = true;
        info!("cleaned result: {:?}", cleaned_result);
        if success {
            return Ok(cleaned_result.join("\n") + "\n");
        } else {
            return Err(SniprunError::RuntimeError("whoops".to_owned()));
        }
    }
}

#[cfg(test)]
mod test_julia_jupyter {
    use super::*;
    use crate::*;

    #[test]
    fn run_all() {
        simple_print();
    }

    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("println(\"a\")");
        let mut interpreter = Julia_jupyter::new(data);
        let res = interpreter.run_at_level(SupportLevel::Bloc);

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert!(string_result.contains(&"a"));
    }
}
