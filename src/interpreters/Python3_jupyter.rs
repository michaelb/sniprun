#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Python3_jupyter {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    imports: String,
    kernel_file: String,
    main_file_path: String,
    launcher_path: String,
    plugin_root: String,
    cache_dir: String,
}

impl Python3_jupyter {
    pub fn fetch_imports(&mut self) -> std::io::Result<()> {
        if self.support_level < SupportLevel::Import {
            return Ok(());
        }
        //no matter if it fails, we should try to run the rest
        let mut file = File::open(&self.data.filepath)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        if !self
            .data
            .current_bloc
            .replace(&[' ', '\t', '\n', '\r'][..], "")
            .is_empty()
        {
            self.code = self.data.current_bloc.clone();
        }
        for line in contents.lines() {
            // info!("lines are : {}", line);
            if line.contains("import ") //basic selection
                && line.trim().chars().next() != Some('#')
            && Python3_jupyter::module_used(line, &self.code)
            {
                // embed in try catch blocs in case uneeded module is unavailable
                self.imports = self.imports.clone() + "\n" + line;
            }
        }
        Ok(())
    }

    fn module_used(line: &str, code: &str) -> bool {
        info!(
            "checking for python module usage: line {} in code {}",
            line, code
        );
        if line.contains("*") {
            return true;
        }
        if line.contains(" as ") {
            if let Some(name) = line.split(" ").last() {
                return code.contains(name);
            }
        }
        for name in line
            .replace(",", " ")
            .replace("from", " ")
            .replace("import ", " ")
            .split(" ")
            .filter(|&x| !x.is_empty())
        {
            if code.contains(name.trim()) {
                return true;
            }
        }
        return false;
    }
}

impl Interpreter for Python3_jupyter {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<Python3_jupyter> {
        //create a subfolder in the cache folder
        let pwd = data.work_dir.clone() + "/python3_jupyter";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&pwd)
            .expect("Could not create directory for python3-jupyter");

        //pre-create string pointing to main file's and binary's path
        let mfp = pwd.clone() + "/main.py";
        let lp = pwd.clone() + "/main.sh";

        let pgr = data.sniprun_root_dir.clone();

        let kp = pwd.clone() + "/kernel_sniprun.json";
        Box::new(Python3_jupyter {
            data,
            support_level: level,
            code: String::new(),
            imports: String::new(),
            kernel_file: kp,
            main_file_path: mfp,
            launcher_path: lp,
            plugin_root: pgr,
            cache_dir: pwd,
        })
    }

    fn get_name() -> String {
        String::from("Python3_jupyter")
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
            String::from("python"),
            String::from("python3"),
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
        let _res = self.fetch_imports();
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
        if !self.imports.is_empty() {
            let mut indented_imports = String::new();
            for import in self.imports.lines() {
                indented_imports = indented_imports + "\t" + import + "\n";
            }

            self.imports = String::from("\ntry:\n") + &indented_imports + "\nexcept:\n\tpass\n";
        }

        self.code = self.imports.clone()
            + "\nprint(\"\")\n"
            + &unindent(&format!("{}{}", "\n", self.code.as_str()));
        Ok(())
    }
    fn build(&mut self) -> Result<(), SniprunError> {
        // info!("python code:\n {}", self.code);
        write(&self.main_file_path, &self.code)
            .expect("Unable to write to file for python3_jupyter");
        Ok(())
    }
    fn execute(&mut self) -> Result<String, SniprunError> {
        let output = Command::new("python3")
            .arg(&self.main_file_path)
            .output()
            .expect("Unable to start process");
        if output.status.success() {
            return Ok(String::from_utf8(output.stdout).unwrap());
        } else {
            return Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr.clone())
                    .unwrap()
                    .lines()
                    .last()
                    .unwrap_or(&String::from_utf8(output.stderr).unwrap())
                    .to_owned(),
            ));
        }
    }
}
impl ReplLikeInterpreter for Python3_jupyter {
    fn fetch_code_repl(&mut self) -> Result<(), SniprunError> {
        self.fetch_code()?;
        let saved_code = self.read_previous_code();
        let mut saved_code: Vec<_> = saved_code.lines().collect();
        if saved_code.is_empty() {
            //initialize kernel. Relying on self.read_previous_code to
            //know when to start a new kernel is important as
            //this will be cleared by the SnipReplMemoryClean command
            let _res = std::fs::remove_file(&self.kernel_file);
            let _res = Command::new("jupyter-kernel")
                .arg("--kernel=python3")
                .arg(String::from("--KernelManager.connection_file=") + &self.kernel_file)
                .spawn();
            info!("Initialized kernel");
        } else {
            // kernel already running
            info!(
                "Using already loaded jupyter kernel at {}",
                self.kernel_file
            );
        }
        // do not re-import already loaded imports
        let mut new_imports = String::new();
        for import in self.imports.lines() {
            if !saved_code.contains(&import) {
                saved_code.push(import);
                new_imports = new_imports + import + "\n";
                info!("new import found: {}", import);
            } else {
                info!("import already loaded: {}", import);
            }
        }

        // save kernel + seen_imports in sniprun memory
        self.save_code(saved_code.join("\n"));

        self.imports = new_imports;

        Ok(())
    }
    fn add_boilerplate_repl(&mut self) -> Result<(), SniprunError> {
        info!("begins add boilerplate repl");
        if !self.imports.is_empty() {
            let mut indented_imports = String::new();
            for import in self.imports.lines() {
                indented_imports = indented_imports + "\t" + import + "\n";
            }

            self.imports = String::from("\ntry:\n") + &indented_imports + "\nexcept:\n\tpass\n";
        }
        //empty print a newline, in case the jupyter prompt interferes.
        //anyway, removed by sniprun itself before display
        self.code = self.imports.clone()
            + "\nprint(\"\")\n"
            + &unindent(&format!("{}{}", "\n", self.code.as_str()));

        Ok(())
    }

    fn build_repl(&mut self) -> Result<(), SniprunError> {
        let actual_command = String::from("echo")
            + " "
            + &String::from("'exec(open(\"")
            + &self.main_file_path
            + "\").read())"
            + "' "
            + "|"
            + " "
            + "jupyter-console"
            + " "
            + "--existing"
            + " "
            + &self.kernel_file.clone()
            + " "
            + "--simple-prompt"
            + " "
            + "-y"
            + " "
            + " --no-confirm"
            + " "
            + "--ZMQTerminalInteractiveShell.banner=\"\""
            + " "
            + "--Application.log_level=0";

        write(&self.launcher_path, &actual_command)
            .expect("Unable to write file for python3_jupyter");
        info!("command written to launcher:\n{}\n", actual_command);
        write(&self.main_file_path, &self.code)
            .expect("Unable to write to file for python3_jupyter");
        Ok(())
    }

    fn execute_repl(&mut self) -> Result<String, SniprunError> {
        let output = Command::new("sh")
            .arg(&self.launcher_path)
            .output()
            .expect("Unable to start process");
        let result = String::from_utf8(output.stdout).unwrap();
        let mut cleaned_result: Vec<_> = result.lines().collect();

        info!("result: {:?}", cleaned_result);

        // first and last lines are the [In] x: prompts from jupyter-console
        cleaned_result.remove(cleaned_result.len() - 1);
        cleaned_result.remove(1);
        cleaned_result.remove(0);

        info!("cleaned result: {:?}", cleaned_result);
        if String::from_utf8(output.stderr.clone()).unwrap().is_empty() {
            return Ok(cleaned_result.join("\n") + "\n");
        } else {
            return Err(SniprunError::RuntimeError(
                String::from_utf8(strip_ansi_escapes::strip(output.stderr.clone()).unwrap())
                    .unwrap()
                    .lines()
                    .last()
                    .unwrap_or(
                        &String::from_utf8(
                            strip_ansi_escapes::strip(output.stderr.clone()).unwrap(),
                        )
                        .unwrap(),
                    )
                    .to_owned(),
            ));
        }
    }
}

#[cfg(test)]
mod test_python3_jupyter {
    use super::*;
    use crate::*;

    #[test]
    fn run_all() {
        simple_print();
    }


    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("print(\"a\",1)");
        let mut interpreter = Python3_jupyter::new(data);
        let res = interpreter.run_at_level(SupportLevel::Bloc);

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert!(string_result.contains(&"a 1"));
    }

    #[test]
    #[cfg_attr(feature = "ignore_in_ci", ignore)]
    #[ignore] // because we don't want to run this in // with simple_print
    fn simple_print_repl() {
        let id = Some(Arc::new(Mutex::new(InterpreterData {
            owner: String::from(""),
            content: String::from(""),
            pid: None,
        })));

        let mut data = DataHolder::new();
        data.repl_enabled = vec![String::from("Python3_jupyter")];
        let mut data2 = DataHolder::new();
        data.interpreter_data = id.clone();
        data2.interpreter_data = id;

        data.current_bloc = String::from("a=1");
        let mut interpreter = Python3_jupyter::new(data2);
        let _res = interpreter.run_at_level_repl(SupportLevel::Import).unwrap();


        data.current_bloc = String::from("print(a)");
        let mut interpreter = Python3_jupyter::new(data);
        let _res = interpreter.run_at_level_repl(SupportLevel::Import);

        // should panic if not an Ok()
        // but for some reason does not work in test mode 
        // let string_result = res.unwrap();
        // assert_eq!(string_result, "1\n");
    }


}
