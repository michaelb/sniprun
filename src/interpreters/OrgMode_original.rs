#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct OrgMode_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,

    default_filetype: String,
}

impl OrgMode_original {
    pub fn get_filetype_of_embbeded_code(&mut self) -> Result<String, SniprunError> {
        let nvim_instance = self.data.nvim_instance.clone().unwrap();
        let mut real_nvim_instance = nvim_instance.lock().unwrap();

        // walk the whole visual selection in case multiple code block are contained
        let lines = real_nvim_instance
            .get_current_buf()
            .unwrap()
            .get_lines(
                &mut real_nvim_instance,
                self.data.range[0] - 1,
                self.data.range[1],
                false,
            )
            .unwrap();
        let mut counter = 0;
        let selection_line = self.data.range[0] as usize;
        let mut v = vec![];
        for (i, l) in lines.iter().enumerate() {
            info!("checking code bloc delimiter in : {}",l);
            if l.trim_start().to_lowercase().starts_with("#+begin_src") {
                if counter % 2 == 1 { return Err(SniprunError::CustomError(String::from("Incomplete or nested code blocs")))} 
                counter += 1;
                v.push((selection_line + i + 1, 0));
            }
            if l.trim_start().to_lowercase().starts_with("#+end_src") {
                if counter % 2 == 0 { return Err(SniprunError::CustomError(String::from("Incomplete or nested code blocs")))} 
                counter += 1;
                v[((counter - 1) / 2) as usize].1 = selection_line + i - 1;
            }
        }
        if counter >= 2 {
            info!("counting {} code blocs delimiters", counter);
            if counter % 2 == 1 {
                return Err(SniprunError::CustomError(String::from(
                    "Selection contains an odd number of code bloc delimiters",
                )));
            }
            info!("running separately ranges : {:?}", v);
            return Err(SniprunError::ReRunRanges(v));
        }
        info!("no muliple bloc was found");

        let mut line_n = self.data.range[0]; // no matter which one

        //first check if we not on boundary of block
        if self.data.current_line.trim_start().to_lowercase().starts_with("#+name")
        {
            let next_line = real_nvim_instance
                .get_current_buf()
                .unwrap()
                .get_lines(&mut real_nvim_instance, line_n, line_n + 1, false)
                .unwrap();
            self.data.current_line = next_line.join("");
            line_n += 1;
        }

        if self
            .data
            .current_line
            .trim_start().to_lowercase()
            .starts_with("#+begin_src")
                {
            let flavor = self
                .data
                .current_line
                .trim_start()
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .to_owned();
            let end_line = real_nvim_instance
                .get_current_buf()
                .unwrap()
                .line_count(&mut real_nvim_instance)
                .unwrap();
            let capped_end_line = std::cmp::min(line_n + 400, end_line); // in case there is a very long file, don't search for nothing up to line 500k
            let it = line_n + 1..capped_end_line + 1;

            let mut code_bloc = vec![];
            for i in it {
                let line_i = real_nvim_instance
                    .get_current_buf()
                    .unwrap()
                    .get_lines(&mut real_nvim_instance, i - 1, i, false)
                    .unwrap()
                    .join("");
                if line_i.trim_start().to_lowercase().starts_with("#+end_src")
                {
                    //found end of bloc
                    self.data.current_bloc = code_bloc.join("\n");
                    info!(
                        "line to extract filetype from: {:?}",
                        line_i.split_whitespace().collect::<Vec<_>>()
                    );
                    return Ok(self.filetype_from_str(&flavor));
                } else {
                    info!("adding line {} to current bloc", i);
                    code_bloc.push(line_i.to_string());
                }
            }
        }

        // if we are in a block
        for i in (1..line_n).rev() {
            {
                let line_i = real_nvim_instance
                    .get_current_buf()
                    .unwrap()
                    .get_lines(&mut real_nvim_instance, i - 1, i, false)
                    .unwrap()
                    .join("");
                if line_i.trim_start().to_lowercase().starts_with("#+begin_src")
                {
                    let flavor = line_i
                        .trim_start()
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("")
                        .to_owned();
                    return Ok(self.filetype_from_str(&flavor));
                }
            }
        }
        Ok(String::new())
    }

    /// Convert orgmode code block flavor to filetype
    pub fn filetype_from_str(&self, s: &str) -> String {
        let cleaned_str = s.replace(&['{', '}', '.'][..], "");
        match cleaned_str.as_str() {
            "bash" => "sh",
            "zsh" => "sh",
            "shell" => "sh",
            "C++" => "cpp",
            "c++" => "cpp",
            "Perl" => "perl",
            "python3" => "python",
            "rb" => "ruby",
            "R" => "r",
            "jruby" => "ruby",
            "objectivec" => "objcpp",
            "ts" => "typescript",
            "" => &self.default_filetype,
            a => a,
        }
        .to_string()
    }
}

impl ReplLikeInterpreter for OrgMode_original {}

impl Interpreter for OrgMode_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Self> {
        //create a subfolder in the cache folder
        let lwd = data.work_dir.clone() + "/orgmode_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&lwd)
            .expect("Could not create directory for example");
        let mut data_clone = data;
        data_clone.work_dir = lwd.clone(); //trick other interpreter at creating their files here

        let ddf = String::from("python"); //default default

        let mut orgmode_interpreter = Box::new(OrgMode_original {
            data: data_clone,
            support_level,
            code: String::new(),
            default_filetype: ddf,
        });

        if let Some(value) = OrgMode_original::get_interpreter_option(
            &orgmode_interpreter.get_data(),
            "default_filetype",
        ) {
            if let Some(valid_string) = value.as_str() {
                orgmode_interpreter.default_filetype = valid_string.to_string();
            }
        }

        orgmode_interpreter
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("OrgMode"),
            String::from("org"),
            String::from("orgmode"),
        ]
    }

    fn get_name() -> String {
        String::from("OrgMode_original")
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
        if !self
            .data
            .current_bloc
            .replace(&[' ', '\t', '\n', '\r'][..], "")
            .is_empty()
            && self.support_level >= SupportLevel::Bloc
        {
            self.code = self.data.current_bloc.clone();
        } else if !self
            .data
            .current_line
            .replace(&[' ', '\t'][..], "")
            .is_empty()
            && self.support_level >= SupportLevel::Line
        {
            //special for orgmode in case we try to run a bloc of markodwn that only has one line,
            //an only Line level support
            self.code = self
                .data
                .current_bloc
                .lines()
                .next()
                .unwrap_or(&String::new())
                .to_string();
        } else {
            // no code was retrieved
            self.code = String::from("");
        }

        self.data.filetype = self.get_filetype_of_embbeded_code()?;
        info!("filetype/flavor found: {}", self.data.filetype);
        info!("Code to run with new filetype: {}", self.data.current_bloc);
        Ok(())
    }

    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        let last_line = self.code.lines().last().unwrap_or("");

        // for some languages, handle an eventual final 'return' as a print
        if last_line.starts_with("return") {
            let printing_last_line = match self.data.filetype.as_str() {
                // after 'fetch, contains the embbeded language filetype
                "python" | "python3" | "py" | "sage.python" => {
                    String::from("print(") + last_line.strip_prefix("return").unwrap() + ")"
                }
                "rust" => {
                    String::from("println!(\"{}\",")
                        + last_line.strip_prefix("return").unwrap()
                        + ")"
                }
                "bash" => String::from("echo ") + last_line.strip_prefix("return").unwrap(),
                _ => last_line.to_string(),
            };
            let mut code_in_lines: Vec<&str> = self.code.lines().collect();
            code_in_lines.pop();
            code_in_lines.push(&printing_last_line);
            self.code = code_in_lines.join("\n");
        }
        Ok(())
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        info!("executing orgmode interpreter");
        let launcher = crate::launcher::Launcher::new(self.data.clone());

        if let Some((name, level)) = launcher.select() {
            info!("Selected real interpreter: {}", name);
            //launch the right interpreter !
            iter_types! {
                if Current::get_name() == name {
                    let mut inter = Current::new_with_level(self.data.clone(), level);
                    return inter.run();
                }
            }
        }
        Err(SniprunError::CustomError(String::from(
            "Failed to determine language of code bloc",
        )))
    }
}

#[cfg(test)]
mod test_orgmode_original {
    use super::*;

    use serial_test::serial;

    #[test]
    #[serial(bash)]
    fn simple_bloc() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("\necho 3");

        data.filetype = String::from("bash");
        data.range = [1, 3];

        let mut interpreter = OrgMode_original::new(data);
        let res = interpreter.execute();
        let string_result = res.unwrap();
        assert_eq!(string_result, "3\n");
    }
}
