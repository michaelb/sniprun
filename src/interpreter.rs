use crate::error::SniprunError;
use crate::DataHolder;
use log::info;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub enum SupportLevel {
    ///no support
    Unsupported = 0,
    ///run the code in the line, all is contained within and no variable declaration/initialisation happens before
    Line = 1,
    ///run a bloc of code, same limitations as Line
    Bloc = 2,
    ///support exterior imports
    Import = 5,
    ///run a line/bloc of code, but include variable/functions definitions found in the file
    File = 10,
    ///run a line/bloc of code, but include (only needed) variable/functions found in the project
    Project = 20,
    ///selected: don't use this support level, it is meant to communicate user's config choices
    Selected = 255,
}

impl Display for SupportLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match *self {
            SupportLevel::Unsupported => f.write_str("None"),
            SupportLevel::Line => f.write_str("Line"),
            SupportLevel::Bloc => f.write_str("Bloc"),
            SupportLevel::Import => f.write_str("Import"),
            SupportLevel::File => f.write_str("File"),
            SupportLevel::Project => f.write_str("Project"),
            SupportLevel::Selected => f.write_str("Selected"),
        }
    }
}

///This is the trait all interpreters must implement.
///The launcher run fucntions new() and run() from this trait.
pub trait Interpreter: ReplLikeInterpreter {
    //create
    fn new(data: DataHolder) -> Box<Self> {
        Self::new_with_level(data, Self::get_max_support_level())
    }
    /// This implies your interpreter struct should have a 'data' and a 'support_level' field.
    /// I suggest you also add a 'code' String field to hold the code you want to modify and run
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<Self>;

    ///Return the (unique) name of your interpreter.
    fn get_name() -> String;

    ///Return whether the interpreter is the default for a given filetype
    fn default_for_filetype() -> bool {
        false
    }

    /// The languages (as filetype codes) supported by your interpreter; check ':set ft?' in neovim
    /// on a file of your language if you are not sure. You can put whatever (python and python3),
    /// but I strongly recommend making the first element the name of the langage ("JavaScript"
    /// instead of "js" for example)
    fn get_supported_languages() -> Vec<String>;

    fn get_current_level(&self) -> SupportLevel;
    fn set_current_level(&mut self, level: SupportLevel);
    fn get_data(&self) -> DataHolder;

    fn get_nvim_pid(data: &DataHolder) -> String {
        // associated utility function
        data.nvim_pid.to_string()
    }

    /// You should override this method as soon as you wish to test your interpreter.
    fn get_max_support_level() -> SupportLevel {
        //to overwrite in trait impls
        SupportLevel::Unsupported
    }

    /// This function should be overwritten if your interpreter cannot run
    /// all the files for the advertised filetypes.
    /// It's up to you to detect it, and initialize (new()) and .run() it and return the result
    fn fallback(&mut self) -> Option<Result<String, SniprunError>> {
        // if incompatible code detected {
        //      let mut good_interpreter =
        //      crate::interpreters::Good_interpreter::new_with_level(&self.data,&self.get_current_level());
        //      return Some(good_interpreter.run());
        //      }
        None
    }

    /// Checks if the interpreter has cli-args support
    /// Can also be used to check the validity of provided args
    fn check_cli_args(&self) -> Result<(), SniprunError> {
        info!("Checking cli-args: {:?}", self.get_data().cli_args);
        if self.get_data().cli_args.is_empty() {
            Ok(())
        } else {
            Err(SniprunError::InterpreterLimitationError(
                "This interpreter does not support command line arguments".to_string(),
            ))
        }
    }

    ///Disable REPL-like behavior by default
    fn behave_repl_like_default() -> bool {
        false
    }

    /// Info only, indicates whether the interpreter has REPL-like behavior
    fn has_repl_capability() -> bool {
        false
    }

    ///If the interpreter has LSP capabilities
    fn has_lsp_capability() -> bool {
        false
    }
    ///
    /// This method should get the needed code from the data struct and eventually the files
    /// of the project
    fn fetch_code(&mut self) -> Result<(), SniprunError>; //mut to allow modification of the current_level

    /// This should add code that does not originate from the project to the code field in the
    /// interpreter
    fn add_boilerplate(&mut self) -> Result<(), SniprunError>;

    /// This should be used to build (compile) the code and produce an executable
    /// this function should be left blank (return Ok(());) for interpreted languages.
    fn build(&mut self) -> Result<(), SniprunError>; //return path to executable

    ///This should be used to execute a binary or execute the script
    ///In case it's successfull, returns Ok( standart_output );
    fn execute(&mut self) -> Result<String, SniprunError>;

    /// set the current support level to the one provided, run fetch(), add_boilerplate(), build() and execute() in order if each step is successfull
    fn run_at_level(&mut self, level: SupportLevel) -> Result<String, SniprunError> {
        self.set_current_level(level);
        let res = self
            .fetch_code()
            .and_then(|_| self.add_boilerplate())
            .and_then(|_| self.build())
            .and_then(|_| self.execute());
        if res.is_err() {
            let alt_res = self.fallback();
            if let Some(Ok(alt_res_ok)) = alt_res {
                return Ok(alt_res_ok);
            }
        }

        res
    }

    fn run_at_level_repl(&mut self, level: SupportLevel) -> Result<String, SniprunError> {
        info!("REPL enabled");
        self.set_current_level(level);
        if let Some(res) = self.fallback() {
            return res;
        }
        self.fetch_code_repl()
            .and_then(|_| self.add_boilerplate_repl())
            .and_then(|_| self.build_repl())
            .and_then(|_| self.execute_repl())
    }

    /// default run function ran from the launcher (run_at_level(max_level))
    fn run(&mut self) -> Result<String, SniprunError> {
        let name = Self::get_name();
        let data = self.get_data();
        // choose whether to use repl-like or normal
        self.check_cli_args()?;

        let decision = (Self::behave_repl_like_default() || data.repl_enabled.contains(&name))
            && !data.repl_disabled.contains(&name);
        if decision {
            self.run_at_level_repl(self.get_current_level())
        } else {
            self.run_at_level(self.get_current_level())
        }
    }
}
#[derive(Debug, PartialEq)]
pub enum ErrTruncate {
    Short,
    Long,
}

pub trait InterpreterUtils {
    ///read previously saved code from the interpreterdata object
    fn read_previous_code(&self) -> String;
    ///append code to the interpreterdata object
    fn save_code(&self, code: String);
    fn clear(&self);

    fn set_pid(&self, pid: u32);
    fn get_pid(&self) -> Option<u32>;
    fn get_interpreter_option(data: &DataHolder, option: &str) -> Option<neovim_lib::Value>;
    fn contains_main(entry: &str, snippet: &str, comment: &str) -> bool;
    fn error_truncate(data: &DataHolder) -> ErrTruncate;
}

impl<T: Interpreter> InterpreterUtils for T {
    ///Read a String previous saved to sniprun memory
    fn read_previous_code(&self) -> String {
        let data = self.get_data();
        info!("reading previous code");
        if data.interpreter_data.is_none() {
            String::new()
        } else {
            info!("found interpreter_data");
            let interpreter_data = data.interpreter_data.unwrap().lock().unwrap().clone();
            let content_owner = T::get_name();
            if interpreter_data.owner == content_owner {
                interpreter_data.content
            } else {
                String::new()
            }
        }
    }

    /// Save an unique String to Sniprun memory.
    /// This will be emptied at neovim startup,
    /// when sniprun is reset or memoryclean'd
    fn save_code(&self, code: String) {
        let previous_code = self.read_previous_code();
        let data = self.get_data();
        if data.interpreter_data.is_none() {
            info!("Unable to save code for next usage");
        } else {
            {
                data.interpreter_data.clone().unwrap().lock().unwrap().owner = T::get_name();
            }
            {
                data.interpreter_data.unwrap().lock().unwrap().content =
                    previous_code + "\n" + &code;
            }
            info!("code saved: {}", self.read_previous_code());
        }
    }

    /// Clear sniprun memory
    fn clear(&self) {
        let data = self.get_data();
        if data.interpreter_data.is_some() {
            data.interpreter_data
                .clone()
                .unwrap()
                .lock()
                .unwrap()
                .owner
                .clear();
            data.interpreter_data
                .unwrap()
                .lock()
                .unwrap()
                .content
                .clear();
        }
    }

    /// save a unsigned int (typically: a pid (of an external processus))
    /// to sniprun memory
    /// This will be emptied at neovim startup,
    /// when sniprun is reset or memoryclean'd
    fn set_pid(&self, pid: u32) {
        if let Some(di) = self.get_data().interpreter_data {
            di.lock().unwrap().pid = Some(pid);
        }
    }

    /// get a unsigned integer previously saved in sniprun memory
    fn get_pid(&self) -> Option<u32> {
        if let Some(di) = self.get_data().interpreter_data {
            di.lock().unwrap().pid
        } else {
            None
        }
    }

    /// get an interpreter option
    fn get_interpreter_option(data: &DataHolder, option: &str) -> Option<neovim_lib::Value> {
        fn index_from_name(
            name: &str,
            config: &[(neovim_lib::Value, neovim_lib::Value)],
        ) -> Option<usize> {
            for (i, kv) in config.iter().enumerate() {
                if name == kv.0.as_str().unwrap_or("") {
                    return Some(i);
                }
            }
            info!("key '{}' not found in interpreter option", name);
            None
        }
        // this is the ugliness required to fetch something from the interpreter options
        if let Some(config) = &data.interpreter_options {
            if let Some(ar) = config.as_map() {
                if let Some(i) = index_from_name("interpreter_options", ar) {
                    if let Some(ar2) = ar[i].1.as_map() {
                        if let Some(i) = index_from_name(&T::get_name(), ar2) {
                            if let Some(interpreter_config) = ar2[i].1.as_map() {
                                if let Some(i) = index_from_name(option, interpreter_config) {
                                    return Some(interpreter_config[i].1.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn error_truncate(data: &DataHolder) -> ErrTruncate {
        if let Some(error_truncate) = T::get_interpreter_option(data, "error_truncate") {
            if let Some(error_truncate) = error_truncate.as_str() {
                info!("Setting truncate to: {}", error_truncate);
                match error_truncate {
                    "short" => return ErrTruncate::Short,
                    "long" => return ErrTruncate::Long,
                    "auto" => {}
                    x => {
                        info!("invalid truncate option: {} (must be 'long', 'short' or 'auto' (default)", x);
                    }
                };
            }
        }
        // auto select
        if data.current_bloc.lines().count() > 4 {
            ErrTruncate::Long
        } else {
            ErrTruncate::Short
        }
    }

    fn contains_main(entry: &str, snippet: &str, comment: &str) -> bool {
        let compact_main: String = entry.split_whitespace().collect();
        let compact_snippet: String = snippet
            .lines()
            .filter(|l| !l.trim().starts_with(comment))
            .collect::<String>()
            .split_whitespace()
            .collect();
        compact_snippet.contains(&compact_main)
    }
}

pub trait ReplLikeInterpreter {
    fn fetch_code_repl(&mut self) -> Result<(), SniprunError> {
        Ok(())
    }
    fn add_boilerplate_repl(&mut self) -> Result<(), SniprunError> {
        Ok(())
    }
    fn build_repl(&mut self) -> Result<(), SniprunError> {
        Ok(())
    }
    fn execute_repl(&mut self) -> Result<String, SniprunError> {
        Err(SniprunError::InterpreterLimitationError(String::from(
            "REPL-like behavior is not implemented for this interpreter",
        )))
    }
}
