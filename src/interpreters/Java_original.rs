#[derive(Clone)]
#[allow(non_camel_case_types)]
pub struct Java_original {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,

    ///specific to java
    java_work_dir: String,
    bin_name: String,
    main_file_path: String,
}
impl ReplLikeInterpreter for Java_original {}
impl Interpreter for Java_original {
    fn new_with_level(data: DataHolder, support_level: SupportLevel) -> Box<Java_original> {
        //create a subfolder in the cache folder
        let jwd = data.work_dir.clone() + "/java_original";
        let mut builder = DirBuilder::new();
        builder.recursive(true);
        builder
            .create(&jwd)
            .expect("Could not create directory for java-original");

        //pre-create string pointing to main file's and binary's path
        let mfp = jwd.clone() + "/Main.java";
        let bn = "Main".to_string(); // remove extension so binary is named 'main'
        Box::new(Java_original {
            data,
            support_level,
            code: String::from(""),
            java_work_dir: jwd,
            bin_name: bn,
            main_file_path: mfp,
        })
    }

    fn get_supported_languages() -> Vec<String> {
        vec![String::from("Java"), String::from("java")]
    }

    fn get_name() -> String {
        String::from("Java_original")
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
        if !Java_original::contains_main("public static void main(", &self.code, "//") {
            self.code = String::from(
                "public class Main {
                public static void main(String[] args) {
                    ",
            ) + &self.code
                + "}
            }";
        }
        Ok(())
    }

    fn build(&mut self) -> Result<(), SniprunError> {
        //write code to file
        let mut _file =
            File::create(&self.main_file_path).expect("Failed to create file for java-original");
        write(&self.main_file_path, &self.code).expect("Unable to write to file for java-original");

        //compile it (to the bin_path that arleady points to the rigth path)
        let output = Command::new("javac")
            .arg("-d")
            .arg(&self.java_work_dir)
            .arg(&self.main_file_path)
            .output()
            .expect("Unable to start process");

        //TODO if relevant, return the error number (parse it from stderr)
        if !output.status.success() {
            Err(SniprunError::CompilationError("".to_string()))
        } else {
            Ok(())
        }
    }

    fn execute(&mut self) -> Result<String, SniprunError> {
        //run th binary and get the std output (or stderr)
        let output = Command::new("java")
            .arg("-cp")
            .arg(&self.java_work_dir)
            .arg(&self.bin_name)
            .output()
            .expect("Unable to start process");
        if output.status.success() {
            Ok(String::from_utf8(output.stdout).unwrap())
        } else if Java_original::error_truncate(&self.get_data()) == ErrTruncate::Short {
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

#[cfg(test)]
mod test_java_original {
    use super::*;

    #[test]
    fn simple_print() {
        let mut data = DataHolder::new();
        data.current_bloc = String::from("System.out.println(\"hello\");");
        let mut interpreter = Java_original::new(data);
        let res = interpreter.run();

        // should panic if not an Ok()
        let string_result = res.unwrap();
        assert_eq!(string_result, "hello\n");
    }
}
