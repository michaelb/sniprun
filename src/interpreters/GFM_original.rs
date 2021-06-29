#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct GFM_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,

    language_work_dir: String,
    default_filetype: String,
}

impl GFM_original {
    pub fn get_filetype_of_embbeded_code(&mut self) -> String {
        let nvim_instance = self.data.nvim_instance.clone().unwrap();
        let mut real_nvim_instance = nvim_instance.lock().unwrap();
        let line_n = self.data.range[0]; // no matter which one

        //first check if we not on boundary of block
        if self.data.current_line.starts_with("```") {
            let flavor = self.data.current_line[3..].to_owned();
            let end_line = real_nvim_instance
                .get_current_buf()
                .unwrap()
                .line_count(&mut real_nvim_instance)
                .unwrap();
            let capped_end_line = std::cmp::min(line_n + 100, end_line); // in case there is a very long file, don't search for nothing up to line 500k
            let it = line_n + 1..capped_end_line + 1;

            let mut code_bloc = vec![];
            for i in it {
                let line_i = real_nvim_instance
                    .get_current_buf()
                    .unwrap()
                    .get_lines(&mut real_nvim_instance, i - 1, i, false)
                    .unwrap()
                    .join("");
                if line_i.starts_with("```") {
                    //found end of bloc
                    self.data.current_bloc = code_bloc.join("\n");
                    return self.filetype_from_str(flavor.trim());
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
                if line_i.starts_with("```") {
                    let ft = line_i[3..].trim().to_owned();
                    return self.filetype_from_str(&ft);
                }
            }
        }
        String::new()
    }

    /// Convert markdowncode block flavor (Github Flavored Markdown) to filetype
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
            "jruby" => "ruby",
            "objectivec" => "objcpp",
            "ts" => "typescript",
            "" => &self.default_filetype,
            a => a,
        }
        .to_string()
    }
}

impl ReplLikeInterpreter for GFM_original {}

impl Interpreter for GFM_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Self> {
       
        //create a subfolder in the cache folder
        let lwd = data.work_dir.clone() + "/gfm_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&lwd)
            .expect("Could not create directory for example");
        let mut data_clone = data.clone();
        data_clone.work_dir = lwd.clone(); //trick other interpreter at creating their files here

        let ddf = String::from("python"); //default default

        let mut gfm_interpreter = Box::new(GFM_original {
            data: data_clone,
            support_level,
            code: String::new(),
            language_work_dir: lwd,
            default_filetype: ddf,
        });


        if let Some(value) = gfm_interpreter.get_interpreter_option("default_filetype") {
            if let Some(valid_string) = value.as_str() {
                gfm_interpreter.default_filetype = valid_string.to_string();
            }
        }

        return gfm_interpreter;

    }

    fn get_supported_languages() -> Vec<String> {
        vec![String::from("Markdown"), String::from("markdown")]
    }

    fn get_name() -> String {
        String::from("GFM_original")
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
            //special for markdown in case we try to run a bloc of markodwn that only has one line,
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

        self.data.filetype = self.get_filetype_of_embbeded_code();
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
        info!("executing markdown interpreter");
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
        return Err(SniprunError::CustomError(String::from(
            "Failed to determine language of code bloc",
        )));
    }
}


#[cfg(test)]
mod test_gfm_original {
    use super::*;

    #[test]
    fn simple_bloc(){
        let mut data = DataHolder::new();
        data.current_bloc = String::from("\necho 3");

        data.filetype = String::from("bash");
        data.range = [1,3];
        
        let mut interpreter = GFM_original::new(data);
        let res = interpreter.execute();
        let string_result = res.unwrap();
        assert_eq!(string_result, "3\n");
    }
}
        
