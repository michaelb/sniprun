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
        Err(SniprunError::CustomError(String::from("Julia_jupyter only works in REPL-enabled mode")))
    }
}
impl ReplLikeInterpreter for Julia_jupyter {
    fn fetch_code_repl(&mut self) -> Result<(), SniprunError> {
        self.fetch_code()?;
        let saved_code = self.read_previous_code();
        if saved_code.is_empty() {
            //initialize kernel. Relying on self.read_previous_code to
            //know when to start a new kernel is important as
            //this will be cleared by the SnipReplMemoryClean command
            let _res = std::fs::remove_file(&self.kernel_file);
            let _res = Command::new("jupyter-kernel")
                .arg("--kernel=julia-1.5")
                .arg(String::from("--KernelManager.connection_file=") + &self.kernel_file)
                .spawn();
            info!("Initialized kernel at {}", self.kernel_file);
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
    fn add_boilerplate_repl(&mut self) -> Result<(), SniprunError> {
        info!("begins add boilerplate repl");
        self.code = unindent(&format!("{}{}", "\n", self.code.as_str()));

        Ok(())
    }

    fn build_repl(&mut self) -> Result<(), SniprunError> {
        Ok(())
    }

    fn execute_repl(&mut self) -> Result<String, SniprunError> {
        info!(
            "json kernel file exists yet? {}",
            std::path::Path::new(&self.kernel_file).exists()
        );
        while !std::path::Path::new(&self.kernel_file).exists() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        std::thread::sleep(std::time::Duration::from_millis(5000));
        info!(
            "json kernel file exists yet? {}",
            std::path::Path::new(&self.kernel_file).exists()
        );

        let mut f = File::create(&self.kernel_file).expect("Unable to read kernel file");
        let mut content = String::new();
        f.read_to_string(&mut content);
        info!("kernel file contents: {}", content);
        info!("kernel file read");
        let client_res = Client::from_reader(f);
        if client_res.is_ok() {
            info!("client is an ok()");
        }else {
            info!("client is not an ok()");
        }
        let client = client_res.unwrap();

        // Spawn an IOPub watcher
        let receiver = client.iopub_subscribe().expect("Unable to subscribe to IOPub watcher");
        std::thread::spawn(move || {
            for msg in receiver {
                info!("Received message from kernel: {:#?}", msg);
            }
        });




        // Command to run
        let command = jupyter_client::commands::Command::Execute {
            code: "print(10)".to_string(),
            silent: false,
            store_history: true,
            user_expressions: HashMap::new(),
            allow_stdin: true,
            stop_on_error: false,
        };

        Ok(String::new())

//         let cleaned_result = String::new();
//         // first and last lines are the [In] x: prompts from jupyter-console
//         cleaned_result.remove(cleaned_result.len() - 1);
//         cleaned_result.remove(1);
//         cleaned_result.remove(0);
//         info!("result: {:?}", cleaned_result);
// 
//         info!("cleaned result: {:?}", cleaned_result);
//         if String::from_utf8(output.stderr.clone()).unwrap().is_empty() {
//             return Ok(cleaned_result.join("\n") + "\n");
//         } else {
//             return Err(SniprunError::RuntimeError(
//                 String::from_utf8(strip_ansi_escapes::strip(output.stderr.clone()).unwrap())
//                     .unwrap()
//                     .lines()
//                     .last()
//                     .unwrap_or(
//                         &String::from_utf8(
//                             strip_ansi_escapes::strip(output.stderr.clone()).unwrap(),
//                         )
//                         .unwrap(),
//                     )
//                     .to_owned(),
//             ));
//         }
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
