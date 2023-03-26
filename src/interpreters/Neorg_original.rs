#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Neorg_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    default_filetype: String,
}

impl Neorg_original {
    pub fn get_filetype_of_embbeded_code(&mut self) -> Result<String, SniprunError> {
        let nvim_instance = self.data.nvim_instance.clone().unwrap();
        let mut real_nvim_instance = nvim_instance.lock().unwrap();

        // walk the whole visual selection in case multiple code block are contained
        // if so, return ReRunRanges(ranges of code blocs) so get_filetype_of_embedded code can
        // return a meaningful answer for each individual code bloc
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
        let mut run_next_code_bloc = 0;
        info!("cli args empty ? {:?}", self.get_data().cli_args.is_empty());
        for (i, l) in lines.iter().enumerate() {
            // rerunranges only named tags if any are asked for
            if self.get_data().cli_args.is_empty()
                || (l.trim_start().to_lowercase().starts_with("#name")
                    && self
                        .get_data()
                        .cli_args
                        .contains(&l.to_lowercase().replace("#name", "").trim().to_string()))
            {
                run_next_code_bloc = 2;
            }

            info!("checking code bloc delimiter in : {}", l);
            if l.trim_start().to_lowercase().starts_with("@code") {
                if run_next_code_bloc == 0 {
                    continue;
                }
                run_next_code_bloc -= 1;
                if counter % 2 == 1 {
                    return Err(SniprunError::CustomError(String::from(
                        "Incomplete or nested code blocs",
                    )));
                }
                counter += 1;
                v.push((selection_line + i + 1, 0));
            }
            if l.trim_start().to_lowercase().starts_with("@end") {
                if run_next_code_bloc == 0 {
                    continue;
                }
                run_next_code_bloc -= 1;
                if counter % 2 == 0 {
                    return Err(SniprunError::CustomError(String::from(
                        "Incomplete or nested code blocs",
                    )));
                }
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

            if v.is_empty() {
                return Err(SniprunError::CustomError("No matching tag #name was found".to_string()));
            }

            info!("running separately ranges : {:?}", v);
            return Err(SniprunError::ReRunRanges(v));
        }

        info!("no multiple bloc was found");

        let mut line_n = self.data.range[0]; // no matter which one

        //first check if we not on boundary of block
        if self
            .data
            .current_line
            .trim_start()
            .to_lowercase()
            .starts_with("#name")
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
            .trim_start()
            .to_lowercase()
            .starts_with("@code")
        {
            let flavor = self
                .data
                .current_line
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
                if line_i.trim_start().to_lowercase().starts_with("@end") {
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
                if line_i.trim_start().to_lowercase().starts_with("@code") {
                    let flavor = line_i
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

    /// Convert neorg code block flavor to filetype
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

impl ReplLikeInterpreter for Neorg_original {}

impl Interpreter for Neorg_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Self> {
        //create a subfolder in the cache folder
        let lwd = data.work_dir.clone() + "/neorg_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&lwd)
            .expect("Could not create directory for example");
        let mut data_clone = data;
        data_clone.work_dir = lwd; //trick other interpreter at creating their files here

        let ddf = String::from("python"); //default default

        let mut neorg_interpreter = Box::new(Neorg_original {
            data: data_clone,
            support_level,
            code: String::new(),
            default_filetype: ddf,
        });

        if let Some(value) = Neorg_original::get_interpreter_option(
            &neorg_interpreter.get_data(),
            "default_filetype",
        ) {
            if let Some(valid_string) = value.as_str() {
                neorg_interpreter.default_filetype = valid_string.to_string();
            }
        }

        neorg_interpreter
    }

    fn get_supported_languages() -> Vec<String> {
        vec![String::from("Neorg"), String::from("norg")]
    }

    fn get_name() -> String {
        String::from("Neorg_original")
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

    fn check_cli_args(&self) -> Result<(), SniprunError> {
        info!("Checking cli-args: {:?}", self.get_data().cli_args);

        // check arguments are #name tags
        let nvim_instance = self.data.nvim_instance.clone().unwrap();
        let mut real_nvim_instance = nvim_instance.lock().unwrap();
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
        for tag_name in self.get_data().cli_args {
            // walk the whole visual selection in case multiple code block are contained

            let mut found = false;
            for (i, l) in lines.iter().enumerate() {
                info!("checking named tag {} in line {}", tag_name, i);
                if l.trim_start().to_lowercase().starts_with("#name")
                    && tag_name.to_lowercase()
                        == (*l.to_lowercase().replace("#name", "").trim())
                {
                    found = true;
                    info!("found named tag {} in line: {}", tag_name, l);
                    break;
                }
            }

            if !found {
                info!("CLI arguments for Neorg should be valid #name tags");
            }
        }

        Ok(())
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
            //special for neorg in case we try to run a bloc of markodwn that only has one line,
            //an only Line level support
            self.code = self
                .data
                .current_bloc
                .lines()
                .next()
                .unwrap_or("")
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
        Ok(())
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        info!("executing neorg interpreter");
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
mod test_neorg_original {
    use super::*;

    use serial_test::serial;

    #[test]
    #[serial(bash)]
    fn simple_bloc() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("\necho 3");

        data.filetype = String::from("bash");
        data.range = [1, 3];

        let mut interpreter = Neorg_original::new(data);
        let res = interpreter.execute();
        let string_result = res.unwrap();
        assert_eq!(string_result, "3\n");
    }
}
