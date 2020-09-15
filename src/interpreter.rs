use crate::error::SniprunError;
use crate::DataHolder;

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
}

///This is the trait all interpreters must implement.
///The launcher run fucntions new() and run() from this trait.
pub trait Interpreter {
    //create
    fn new(data: DataHolder) -> Box<Self> {
        Self::new_with_level(data, Self::get_max_support_level())
    }
    /// This implies your interpreter struct should have a 'data' and a 'support_level' field.
    /// I suggest you also add a 'code' String field to hold the code you want to modify and run
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<Self>;

    ///Return the (unique) name of your interpreter.
    fn get_name() -> String;

    /// The languages (as filetype codes) supported by your interpreter; check ':set ft?' in neovim
    /// on a file of your language if you are not sure
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
    /// default run function ran from the launcher (run_at_level(max_level))
    fn run(&mut self) -> Result<String, SniprunError> {
        self.run_at_level(self.get_current_level())
    }
}
