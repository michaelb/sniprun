#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Markdown_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,

    language_work_dir: String,
}

impl Markdown_original {
    pub fn get_filetype_of_embbeded_code(&mut self) -> String {
        let nvim_instance = self.data.nvim_instance.clone().unwrap();
        let mut real_nvim_instance = nvim_instance.lock().unwrap();
        let line_n = self.data.range[0]; // no matter which one


        //first check if we not on boundary of block
        info!("current line : {}", self.data.current_line);
        if self.data.current_line.starts_with("```") {
            return String::new();
            //do nothing
            //
            //TODO in the future, maybe run the whole bloc if sniprun on boundary
        }


        for i in (1..line_n).rev() {
            {
                let line_i = real_nvim_instance
                    .get_current_buf()
                    .unwrap()
                    .get_lines(&mut real_nvim_instance, i - 1, i, false)
                    .unwrap()
                    .join("");
                info!("line tried: {}", line_i);
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
        match s {
            "c" => String::from("c"),
            "python" => String::from("python"),
            "bash" => String::from("sh"),
            "sh" => String::from("sh"),
            "zsh" => String::from("sh"),
            "shell" => String::from("sh"),
            "clojure" => String::from("clojure"),
            "cpp" => String::from("cpp"),
            "C++" => String::from("cpp"),
            "cs" => String::from("cs"),
            "elixir" => String::from("elixir"),
            "go" => String::from("go"),
            "java" => String::from("java"),
            "javascript" => String::from("javascript"),
            "Perl" => String::from("perl"),
            "perl" => String::from("perl"),
            "python3" => String::from("python"),
            "rust" => String::from("rust"),
            "rb" => String::from("ruby"),
            "jruby" => String::from("ruby"),
            "ruby" => String::from("ruby"),
            "csharp" => String::from("csharp"),
            "haskell" => String::from("haskell"),
            "matlab" => String::from("matlab"),
            "objectivec" => String::from("objcpp"),
            "swift" => String::from("swift"),
            &_ => String::new(),
        }
    }
}

impl ReplLikeInterpreter for Markdown_original {}

impl Interpreter for Markdown_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Self> {
        //create a subfolder in the cache folder
        let lwd = data.work_dir.clone() + "/markdown_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&lwd)
            .expect("Could not create directory for example");
        let mut data_clone = data.clone();
        data_clone.work_dir = lwd.clone(); //trick other interpreter at creating their files here

        Box::new(Markdown_original {
            data: data_clone,
            support_level,
            code: String::new(),
            language_work_dir: lwd,
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![String::from("Markdown"), String::from("markdown")]
    }

    fn get_name() -> String {
        String::from("Markdown_original")
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
            self.code = self.data.current_line.clone();
        } else {
            // no code was retrieved
            self.code = String::from("");
        }

        self.data.filetype = self.get_filetype_of_embbeded_code();
        info!("filetype found: {}", self.data.filetype);
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
                    let mut inter = Current::new_with_level(self.data.clone(), level); //level limited to Bloc
                    return inter.run();
                }
            }

        }
        return Err(SniprunError::CustomError(String::from("Failed to determine language of code bloc")));
    }
}
