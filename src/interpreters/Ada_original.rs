#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Ada_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,

    ///specific to compiled languages, can be modified of course
    ada_work_dir: String,
    bin_path: String,
    main_file_path: String,
    // you can and should add fields as needed
}

//necessary boilerplate, you don't need to implement that if you want a Bloc support level
//interpreter (the easiest && most common)
impl ReplLikeInterpreter for Ada_original {}

impl Interpreter for Ada_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Ada_original> {
        //create a subfolder in the cache folder
        let awd = data.work_dir.clone() + "/ada_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&awd)
            .expect("Could not create directory for example");

        //pre-create string pointing to main file's and binary's path
        let mfp = awd.clone() + "/main.adb";
        let bp = awd.clone() + "/main"; // remove extension so binary is named 'main'
        Box::new(Ada_original {
            data,
            support_level,
            code: String::new(),
            ada_work_dir: awd,
            bin_path: bp,
            main_file_path: mfp,
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![
            String::from("Ada"), // in 1st position of vector, used for info only
            String::from("ada"),
        ]
    }

    fn get_name() -> String {
        // get your interpreter name
        String::from("Ada_original")
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
        SupportLevel::Line
    }

    fn default_for_filetype() -> bool {
        true
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
        } else if !self.data.current_line.replace(' ', "").is_empty()
            && self.support_level >= SupportLevel::Line
        {
            self.code = self.data.current_line.clone();
        } else {
            self.code = String::from("");
        }
        Ok(())
    }

    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        self.code = String::from(
            "with Ada.Text_IO;\nuse Ada.Text_IO;\nprocedure main is\n\nbegin\n",
        ) + &self.code
            + "\nend main;";
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        //write code to file
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for language_subname");
        write(&self.main_file_path, &self.code)
            .expect("Unable to write to file for language_subname");

        let output = Command::new("gnatmake")
            .arg("main")
            .arg(&self.main_file_path)
            .current_dir(&self.ada_work_dir)
            .output()
            .expect("Unable to start process");
        if !output.status.success() {
            return Err(SniprunError::CompilationError(
                String::from_utf8(output.stderr).unwrap(),
            ));
        }

        Ok(())
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        let output = Command::new(&self.bin_path)
            .output()
            .expect("Unable to start process");

        if output.status.success() {
            //return stdout
            Ok(String::from_utf8(output.stdout).unwrap())
        } else {
            // return stderr
            Err(SniprunError::RuntimeError(
                String::from_utf8(output.stderr).unwrap(),
            ))
        }
    }
}

// You can add tests if you want to
#[cfg(test)]
mod test_ada_original {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial(ada)]
    fn simple_print() {
        let mut data = DataHolder::new();

        data.current_line = String::from("Put_Line(\"Hi\");");
        let mut interpreter = Ada_original::new(data);
        let res = interpreter.run();

        // -> should panic if not an Ok()
        let string_result = res.unwrap();

        // -> compare result with predicted
        assert_eq!(string_result, "Hi\n");
    }
}
