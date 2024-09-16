use crate::input;
use crate::interpreters::import::*;

#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct SQL_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,

    main_file_path: String,
}
impl ReplLikeInterpreter for SQL_original {}
impl Interpreter for SQL_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<SQL_original> {
        //create a subfolder in the cache folder
        let rwd = data.work_dir.clone() + "/sql_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&rwd)
            .expect("Could not create directory for sql-original");

        //pre-create string pointing to main file's and binary's path
        let mfp = rwd + "/main.sql";
        Box::new(SQL_original {
            data,
            support_level,
            code: String::from(""),
            main_file_path: mfp,
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("SQL"),
            String::from("sql"),
            String::from("usql"),
        ]
    }

    fn get_name() -> String {
        String::from("SQL_original")
    }

    fn get_current_level(&self) -> SupportLevel {
        self.support_level
    }
    fn set_current_level(&mut self, level: SupportLevel) {
        self.support_level = level;
    }

    fn default_for_filetype() -> bool {
        true
    }

    fn get_data(&self) -> DataHolder {
        self.data.clone()
    }

    fn get_max_support_level() -> SupportLevel {
        SupportLevel::Bloc
    }

    fn fetch_code(&mut self) -> Result<(), SniprunError> {
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
            self.code = String::from("");
        }

        if self.read_previous_code().is_empty() {
            if let Some(nvim_instance) = self.data.nvim_instance.clone() {
                let user_input =
                    input::vim_input_ask("Enter uSQL database address:", &nvim_instance)?;
                self.save_code(user_input);
            }
        }
        Ok(())
    }

    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        //write code to file
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for sql-original");
        write(&self.main_file_path, &self.code).expect("Unable to write to file for sql-original");
        Ok(())
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        //run th binary and get the std output (or stderr)
        let interpreter = SQL_original::get_interpreter_or(&self.data, "usql");
        let output = Command::new(interpreter.split_whitespace().next().unwrap())
            .args(interpreter.split_whitespace().skip(1))
            .arg("-w")
            .arg("--file")
            .arg(&self.main_file_path)
            .arg(self.read_previous_code().replace('\n', "")) // contains database address
            .current_dir(&self.data.projectroot)
            .output()
            .expect("Unable to start process");
        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else if SQL_original::error_truncate(&self.get_data()) == ErrTruncate::Short {
            Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr.clone())
                    .unwrap()
                    .lines()
                    .next()
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
