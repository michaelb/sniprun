#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Prolog_swi {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
    prolog_work_dir: String,
    main_file_path: String,
}
impl ReplLikeInterpreter for Prolog_swi {}
impl Interpreter for Prolog_swi {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<Prolog_swi> {
        let bwd = data.work_dir.clone() + "/prolog-swi";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&bwd)
            .expect("Could not create directory for prolog-swi");
        let mfp = bwd.clone() + "/main.pl";
        Box::new(Prolog_swi {
            data,
            support_level: level,
            code: String::from(""),
            prolog_work_dir: bwd,
            main_file_path: mfp,
        })
    }
    fn get_name() -> String {
        String::from("Prolog_swi")
    }
    fn get_supported_languages() -> Vec<String> {
        vec![String::from("Prolog"), String::from("prolog")]
    }
    fn get_current_level(&self) -> SupportLevel {
        self.support_level
    }
    fn set_current_level(&mut self, level: SupportLevel) {
        self.support_level = level
    }
    fn default_for_filetype() -> bool {
        true
    }
    fn get_data(&self) -> DataHolder {
        self.data.clone()
    }
    fn check_cli_args(&self) -> Result<(), SniprunError> {
        // All cli arguments are sendable to python
        // Though they will be ignored in REPL mode
        Ok(())
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
        Ok(())
    }
    fn build(&mut self) -> Result<(), SniprunError> {
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for prolog-swi");

        write(&self.main_file_path, &self.code).expect("Unable to write to file for prolog-swi");
        Ok(())
    }
    fn execute(&mut self) -> Result<String, SniprunError> {
        let output = Command::new("swipl")
            .arg(&self.main_file_path)
            .args(&self.get_data().cli_args)
            .output()
            .expect("Unable to start process");
        info!("yay from SWI Prolog interpreter");
        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else {
            Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr).unwrap(),
            ))
        }
    }
}
#[cfg(test)]
mod test_prolog_swi {
    use super::*;

    #[test]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from(":- write(ok), halt.");
        let mut interpreter = Prolog_swi::new(data);
        let res = interpreter.run();

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert_eq!(string_result, "ok");
    }
}
