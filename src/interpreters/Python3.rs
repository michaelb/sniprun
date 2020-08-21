use pyo3::types::PyDict;
use unindent::unindent;

#[derive(Debug, Clone)]
pub struct Python3 {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
}

impl Interpreter for Python3 {
    fn new_with_level(data: DataHolder, level: SupportLevel) -> Box<Python3> {
        Box::new(Python3 {
            data,
            support_level: level,
            code: String::from(""),
        })
    }

    fn get_name() -> String {
        String::from("python3-original")
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
        SupportLevel::Bloc
    }

    fn fetch_code(&mut self) -> Result<(), SniprunError> {
        if !self
            .data
            .current_bloc
            .replace(&[' ', '\t', '\n', '\r'][..], "")
            .is_empty()
        {
            self.code = self.data.current_bloc.clone();
        } else if !self.data.current_line.replace(" ", "").is_empty() {
            self.code = self.data.current_line.clone();
        } else {
            self.code = String::from("");
        }
        Ok(())
    }
    fn add_boilerplate(&mut self) -> Result<(), SniprunError> {
        self.code = String::from(
            "from io import StringIO
import sys

sys.stdout = mystdout1427851999 = StringIO()

",
        ) + &unindent(&format!("{}{}", "\n", self.code.as_str()))
            + "
exit_value1428571999 = str(mystdout1427851999.getvalue())";
        Ok(())
    }
    fn build(&mut self) -> Result<(), SniprunError> {
        Ok(())
    }
    fn execute(&mut self) -> Result<String, SniprunError> {
        let py = pyo3::Python::acquire_gil();
        let locals = PyDict::new(py.python());
        match py.python().run(self.code.as_str(), None, Some(locals)) {
            Ok(_) => (),
            Err(e) => {
                return Err(SniprunError::InterpreterError);
            }
        }
        let py_stdout = locals.get_item("exit_value1428571999");
        if let Some(unwrapped_stdout) = py_stdout {
            let result: Result<String, _> = unwrapped_stdout.extract();
            match result {
                Ok(unwrapped_result) => return Ok(unwrapped_result),
                Err(e) => return Err(SniprunError::InterpreterError),
            }
        } else {
            return Err(SniprunError::InterpreterLimitationError(String::from(
                "Code erased a needed value to get standart output)",
            )));
        }
    }
}
