use crate::interpreter::{Interpreter, SupportLevel};
use crate::DataHolder;

use pyo3::{
    prelude::*,
    types::{PyBytes, PyDict},
};

pub struct Python3 {
    support_level: SupportLevel,
    data: DataHolder,
    code: String,
}

impl Interpreter for Python3 {
    fn new_with_level<'a>(data: DataHolder, level: SupportLevel) -> &'a Python3 {
        &Python3 {
            data,
            support_level: level,
            code: String::from(""),
        }
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
        self.data
    }

    fn get_max_support_level() -> SupportLevel {
        SupportLevel::Unsupported
    }

    fn fetch_code(&mut self) {
        self.code = String::from("print(\"this is a success\")");
    }
    fn add_boilerplate(&mut self) {
        self.code = String::from(
            "from io import StringIO
import sys

sys.stdout = mystdout1427851999 = StringIO()

",
        ) + self.code.as_str()
            + "
exit_value1428571999 = str(mystdout1427851999.getvalue())";
    }
    fn build(&mut self) {}
    fn execute(&mut self) -> Result<String, String> {
        let py = pyo3::Python::acquire_gil().python();
        let locals = PyDict::new(py);
        py.run(self.code.as_str(), None, Some(locals)).unwrap();
        let py_stdout = locals.get_item("exit_value1428571999").unwrap();
        let result: String = py_stdout.extract().unwrap();
        Ok(result)
    }
}
