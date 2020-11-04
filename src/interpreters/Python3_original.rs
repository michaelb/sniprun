//Interpreter:| Python3_original    | python3     |
//############|_____________________|_____________|________________<- delimiters to help formatting,
//###########| Interpretername      | language    | comment
// Keep (but modify the first line after the :) if you wish to have this interpreter listedvia SnipList
#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Python3_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    imports: String,
    main_file_path: String,
    plugin_root: String,
    cache_dir: String,
}

fn module_used(line: &str, code: &str) -> bool {
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
    {
        if code.contains(name.trim()) {
            return true;
        }
    }
    return false;
}

impl Python3_original {
    pub fn fetch_imports(&mut self) -> std::io::Result<()> {
        if self.support_level < SupportLevel::Import {
            return Ok(());
        }
        //no matter if it fails, we should try to run the rest
        let mut file = File::open(&self.data.filepath)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        for line in contents.lines() {
            // info!("lines are : {}", line);
            if line.contains("import ") //basic selection
                && line.trim().chars().next() != Some('#')
            && module_used(line, &contents)
            {
                // embed in try catch blocs in case uneeded module is unavailable
                self.imports = self.imports.clone()
                    + "\n
try:\n" + "\t" + line
                    + "\nexcept:\n\t"
                    + "pass\n";
            }
        }
        Ok(())
    }
}

impl Interpreter for Python3_original {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<Python3_original> {
        //create a subfolder in the cache folder
        let rwd = data.work_dir.clone() + "/python3_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&rwd)
            .expect("Could not create directory for python3-original");

        //pre-create string pointing to main file's and binary's path
        let mfp = rwd.clone() + "/main.py";

        let pgr = data.sniprun_root_dir.clone();
        Box::new(Python3_original {
            data,
            support_level: level,
            code: String::from(""),
            imports: String::from(""),
            main_file_path: mfp,
            plugin_root: pgr,
            cache_dir: rwd,
        })
    }

    fn get_name() -> String {
        String::from("Python3_original")
    }

    fn behave_repl_like_default() -> bool {
        true
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
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
        self.code = self.imports.clone() + &unindent(&format!("{}{}", "\n", self.code.as_str()));
        Ok(())
    }
    fn build(&mut self) -> Result<(), SniprunError> {
        // info!("python code:\n {}", self.code);
        write(&self.main_file_path, &self.code)
            .expect("Unable to write to file for python3_original");
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
impl ReplLikeInterpreter for Python3_original {
    fn fetch_code_repl(&mut self) -> Result<(), SniprunError> {
        self.fetch_code()
    }
    fn build_repl(&mut self) -> Result<(), SniprunError> {
        self.build()
    }

    fn execute_repl(&mut self) -> Result<String, SniprunError> {
        self.execute()
    }
    fn add_boilerplate_repl(&mut self) -> Result<(), SniprunError> {
        info!("begins add boilerplate repl");
        //load save & load functions
        let mut path_to_python_functions = self.plugin_root.clone();
        path_to_python_functions.push_str("/src/interpreters/Python3_original/saveload.py");
        let python_functions = std::fs::read_to_string(&path_to_python_functions).unwrap();
        let klepto_memo = String::from("'") + &self.cache_dir.clone() + "/" + "memo" + "'";

        let mut final_code = self.imports.clone();
        final_code.push_str("\n");
        final_code.push_str(&python_functions);
        final_code.push_str("\n");
        if self.read_previous_code().is_empty() {
            //first run
            self.save_code("Not the first run anymore".to_string());
        } else {
            //not the first run, should load old variables
            {
                final_code.push_str("sniprun142859_load(");
                final_code.push_str(&klepto_memo);
                final_code.push_str(")");
            }
            final_code.push_str("\n");
        }

        final_code.push_str(&unindent(&format!("{}{}", "\n", self.code.as_str())));
        final_code.push_str("\n");
        {
            final_code.push_str("sniprun142859_save("); // if the run has not failed, save new variables
            final_code.push_str(&klepto_memo);
            final_code.push_str(")");
        }

        self.code = final_code.clone();
        // info!("---{}---", &final_code);

        Ok(())
    }

    // &unindent(&format!("{}{}", "\n", self.code.as_str()));
}
