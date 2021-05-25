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
    ///Run a line/bloc of code, but include variable/function from the project and project or system-wide dependencies
    System = 30,
    ///selected: don't use this support level, it is meant to communicate user's config choices
    Selected = 255,
}

impl Display for SupportLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match *self {
            SupportLevel::Unsupported => f.write_str("Unsupported"),
            SupportLevel::Line => f.write_str("Line"),
            SupportLevel::Bloc => f.write_str("Bloc"),
            SupportLevel::Import => f.write_str("Import"),
            SupportLevel::File => f.write_str("File"),
            SupportLevel::Project => f.write_str("Project"),
            SupportLevel::System => f.write_str("System"),
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

    /// You should override this method as soon as you wish to test your interpreter.
    fn get_max_support_level() -> SupportLevel {
        //to overwrite in trait impls
        return SupportLevel::Unsupported;
    }

    /// This function should be overwritten if your intepreter cannot run
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

    ///Disable REPL-like behavior by default
    fn behave_repl_like_default() -> bool {
        false
    }

    /// Info only, indicates whether the interpreter has REPL-like behavior
    fn has_repl_capability() -> bool {
        false
    }

    ///If the interpreter has treesitter capabilities
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
        if let Some(res) = self.fallback() {
            return res;
        }
        self.fetch_code()
            .and_then(|_| self.add_boilerplate())
            .and_then(|_| self.build())
            .and_then(|_| self.execute())
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
        let decision = (Self::behave_repl_like_default() || data.repl_enabled.contains(&name))
            && !data.repl_disabled.contains(&name);
        if decision {
            self.run_at_level_repl(self.get_current_level())
        } else {
            self.run_at_level(self.get_current_level())
        }
    }
}

pub trait InterpreterUtils {
    ///read previously saved code from the interpreterdata object
    fn read_previous_code(&self) -> String;
    ///append code to the interpreterdata object
    fn save_code(&self, code: String);
    fn clear(&self);

    fn set_pid(&self, pid: u32);
    fn get_pid(&self) -> Option<u32>;
    fn get_interpreter_option(&self, option: &str) -> Option<neovim_lib::Value>;
}

impl<T: Interpreter> InterpreterUtils for T{
    ///Read a String previous saved to sniprun memory
    fn read_previous_code(&self) -> String {
        let data = self.get_data();
        if data.interpreter_data.is_none() {
            return String::new();
        } else {
            let interpreter_data = data.interpreter_data.unwrap().lock().unwrap().clone();
            let content_owner = T::get_name();
            if interpreter_data.owner == content_owner {
                return interpreter_data.content;
            } else {
                return String::new();
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
            return;
        } else {
            {
                data.interpreter_data.clone().unwrap().lock().unwrap().owner = T::get_name();
            }
            data.interpreter_data.unwrap().lock().unwrap().content = previous_code + "\n" + &code;
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

    /// save a pid (of an external processus)
    /// to sniprun memory
    /// This will be emptied at neovim startup,
    /// when sniprun is reset or memoryclean'd
    fn set_pid(&self, pid: u32) {
        if let Some(di) = self.get_data().interpreter_data {
            di.lock().unwrap().pid = Some(pid);
        }
    }

    /// get a pid previously saved in sniprun memory
    fn get_pid(&self) -> Option<u32> {
        if let Some(di) = self.get_data().interpreter_data {
            if let Some(real_pid) = di.lock().unwrap().pid {
                return Some(real_pid);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }


    /// get an intepreter option
    fn get_interpreter_option(&self, option:&str) -> Option<neovim_lib::Value> {
        fn index_from_name(
            name: &str,
            config: &Vec<(neovim_lib::Value, neovim_lib::Value)>,
        ) -> Option<usize> {
            for (i, kv) in config.iter().enumerate() {
                if name == kv.0.as_str().unwrap() {
                    return Some(i);
                }
            }
            info!("key '{}' not found in interpreter option", name);
            return None;
        }
        // this is the ugliness required to fetch something from the interpreter options
        if let Some(config) = &self.get_data().interpreter_options {
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
            "REPL-like behavior is not implemented for this intepreter",
        )))
    }
}
