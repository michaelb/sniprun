#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Generic {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    workdir: String,
    interpreted_lang: bool,
    interpreter: String,
    compiler: String,
    exe_path: String,
    main_file_path: String,
    boilerplate_pre: String,
    boilerplate_post: String,
}
impl Generic {
    // Custom fetch function for the generic interpreter
    fn generic_get_interpreter_option(
        data: &DataHolder,
        option: &str,
    ) -> Option<(String, neovim_lib::Value)> {
        fn index_from_name(
            name: &str,
            config: &[(neovim_lib::Value, neovim_lib::Value)],
        ) -> Option<usize> {
            for (i, kv) in config.iter().enumerate() {
                if name == kv.0.as_str().unwrap_or("") {
                    info!("key {} found", name);
                    return Some(i);
                }
            }
            info!("key '{}' not found in interpreter option", name);
            None
        }

        fn config_supports_filetype(
            ft: &str,
            config: &[(neovim_lib::Value, neovim_lib::Value)],
        ) -> bool {
            if let Some(i) = index_from_name("supported_filetypes", config) {
                let supported_filetypes_string_or_map = &config[i].1; // let's accept strings for convenience' sake
                                                                      // if key was unique filetype as string
                if let Some(supported_filetype_str) = supported_filetypes_string_or_map.as_str() {
                    if supported_filetype_str == ft {
                        return true;
                    }
                }
                // if key was multiple filetypes as array
                if let Some(supported_filetype_ar) = supported_filetypes_string_or_map.as_array() {
                    for v in supported_filetype_ar.iter() {
                        if ft == v.as_str().unwrap_or("") {
                            info!("key {} found", ft);
                            return true;
                        }
                    }
                }
                info!(
                    "supported filetypes neither array or string ? {:?}",
                    supported_filetypes_string_or_map
                );
            } else {
                info!("Generic configs should always specify 'supported_filetypes'");
            }

            false
        }

        // this is the ugliness required to fetch something from the Generic interpreter options
        if let Some(config) = &data.interpreter_options {
            if let Some(ar) = config.as_map() {
                if let Some(i) = index_from_name("interpreter_options", ar) {
                    if let Some(ar2) = ar[i].1.as_map() {
                        if let Some(i) = index_from_name(&Generic::get_name(), ar2) {
                            if let Some(ar3) = ar2[i].1.as_map() {
                                for kv in ar3.iter() {
                                    info!(
                                        "checking if generic config {} matches filetypes",
                                        kv.0.as_str().unwrap_or("toostrangeconfigname")
                                    );
                                    // iter on all config in Generic
                                    if let Some(ar_config) = kv.1.as_map() {
                                        if config_supports_filetype(&data.filetype, ar_config) {
                                            if let Some(i) = index_from_name(option, ar_config) {
                                                return Some((
                                                    kv.0.as_str()
                                                        .unwrap_or("wrong_key_name")
                                                        .to_string(),
                                                    ar_config[i].1.clone(),
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }
}

impl ReplLikeInterpreter for Generic {}
impl Interpreter for Generic {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Generic> {
        let mut interpreter = String::new();
        let mut compiler = String::new();
        let mut interpreted_lang = false;
        let mut config_name = String::from("unnamed");
        let mut exe_name = String::from("a.out");
        let mut extension = String::from("");
        let mut boilerplate_pre = String::from("");
        let mut boilerplate_post = String::from("");

        if let Some((fetched_config_name, used_compiler)) =
            Generic::generic_get_interpreter_option(&data, "compiler")
        {
            if let Some(compiler_string) = used_compiler.as_str() {
                info!("Using compiler: {}", compiler_string);
                compiler = compiler_string.to_string();
                config_name = fetched_config_name;
                interpreted_lang = false;
            }
        }

        if let Some((fetched_config_name, used_interpreter)) =
            Generic::generic_get_interpreter_option(&data, "interpreter")
        {
            if let Some(interpreter_string) = used_interpreter.as_str() {
                info!("Using interpreter: {}", interpreter_string);
                interpreter = interpreter_string.to_string();
                config_name = fetched_config_name;
                interpreted_lang = true;
            }
        }

        if interpreter.is_empty() && compiler.is_empty() {
            info!("neither compiler or interpreter was provided in generic options!");
        }

        if let Some((_, used_exe_name)) = Generic::generic_get_interpreter_option(&data, "exe_name")
        {
            if let Some(exe_name_string) = used_exe_name.as_str() {
                info!("Using exe_name: {}", exe_name_string);
                exe_name = exe_name_string.to_string();
            }
        }

        if let Some((_, used_extension)) =
            Generic::generic_get_interpreter_option(&data, "extension")
        {
            if let Some(extension_string) = used_extension.as_str() {
                info!("Using extension: {}", extension_string);
                extension = extension_string.to_string();
            }
        }

        if let Some((_, used_boilerplate_pre)) =
            Generic::generic_get_interpreter_option(&data, "boilerplate_pre")
        {
            if let Some(boilerplate_pre_string) = used_boilerplate_pre.as_str() {
                info!("Using boilerplate_pre: {}", boilerplate_pre_string);
                boilerplate_pre = boilerplate_pre_string.to_string();
            }
        }
        if let Some((_, used_boilerplate_post)) =
            Generic::generic_get_interpreter_option(&data, "boilerplate_post")
        {
            if let Some(boilerplate_post_string) = used_boilerplate_post.as_str() {
                info!("Using boilerplate_post: {}", boilerplate_post_string);
                boilerplate_post = boilerplate_post_string.to_string();
            }
        }

        let rwd = data.work_dir.clone() + "/generic/" + &config_name;
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&rwd)
            .expect("Could not create directory for generic/<config_name>");
        let exe_path = rwd.clone() + "/" + &exe_name;
        let main_file_path = exe_path.clone() + "_src123456789src" + &extension; // this way, virtually impossible for the user for have exe & main file with same name by accident
        Box::new(Generic {
            data,
            support_level,
            code: String::from(""),
            exe_path,
            interpreted_lang,
            interpreter,
            compiler,
            workdir: rwd,
            main_file_path,
            boilerplate_pre,
            boilerplate_post,
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![]
    }

    fn get_name() -> String {
        String::from("Generic")
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
        //actually this has no importance since we're already in the 'fallback' generic interpreter
    }

    fn fetch_code(&mut self) -> Result<(), SniprunError> {
        if !self
            .data
            .current_bloc
            .replace(&[' ', '\t', '\n', '\r'][..], "")
            .is_empty()
        {
            self.code = self.data.current_bloc.clone();
        } else if !self.data.current_line.replace(' ', "").is_empty() {
            self.code = self.data.current_line.clone();
        } else {
            self.code = String::from("");
        }
        if self.interpreter.is_empty() && self.compiler.is_empty() {
            return Err(SniprunError::CustomError(
                "Filetype not officially supported, nor configured for the Generic interpreter"
                    .to_string(),
            ));
        }
        Ok(())
    }

    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        self.code =
            self.boilerplate_pre.clone() + "\n" + &self.code + "\n" + &self.boilerplate_post;
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        if self.interpreted_lang {
            let mut _file =
                File::create(&self.main_file_path).expect("Failed to create file for generic");
            write(&self.main_file_path, &self.code).expect("Unable to write to file for generic");
        } else {
            let mut _file =
                File::create(&self.main_file_path).expect("Failed to create file for generic");
            write(&self.main_file_path, &self.code).expect("Unable to write to file for generic");
            info!("compiling main file to exe");

            let output = Command::new(self.compiler.split_whitespace().next().unwrap())
                .args(self.compiler.split_whitespace().skip(1))
                .arg(&self.main_file_path)
                .current_dir(&self.workdir)
                .output()
                .expect("Unable to execute compiler");

            info!(
                "generic compiled, status.success?:{}",
                output.status.success()
            );
            if output.status.success() {
                return Ok(());
            } else if Generic::error_truncate(&self.get_data()) == ErrTruncate::Short {
                return Err(SniprunError::CompilationError(
                    String::from_utf8(output.stderr.clone())
                        .unwrap()
                        .lines()
                        .last()
                        .unwrap_or(&String::from_utf8(output.stderr).unwrap())
                        .to_owned(),
                ));
            } else {
                return Err(SniprunError::CompilationError(
                    String::from_utf8(output.stderr).unwrap(),
                ));
            }
        }
        Ok(())
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        let output = if self.interpreted_lang {
            Command::new(self.interpreter.split_whitespace().next().unwrap())
                .args(self.interpreter.split_whitespace().skip(1))
                .arg(&self.main_file_path)
                .args(&self.get_data().cli_args)
                .current_dir(&self.workdir)
                .output()
                .expect("Unable to start process")
        } else {
            Command::new(self.exe_path.clone())
                .args(&self.get_data().cli_args)
                .current_dir(&self.workdir)
                .output()
                .expect("Unable to start process")
        };
        info!(
            "generic executed, status.success?:{}",
            output.status.success()
        );
        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else if Generic::error_truncate(&self.get_data()) == ErrTruncate::Short {
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
