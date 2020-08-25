use crate::*;
use error::SniprunError;
use interpreter::{Interpreter, SupportLevel};

pub struct Launcher {
    pub data: DataHolder,
}

impl Launcher {
    pub fn new(data: DataHolder) -> Self {
        Launcher { data: data }
    }

    pub fn select_and_run<'a>(&self) -> Result<String, SniprunError> {
        let mut max_level_support = SupportLevel::Unsupported;
        let mut name_best_interpreter = String::from("Generic");
        //select the best interpreter for the language
        iter_types! {
            if Current::get_supported_languages().contains(&self.data.filetype){
                if Current::get_max_support_level() > max_level_support {
                    max_level_support = Current::get_max_support_level();
                    name_best_interpreter = Current::get_name();
                }
            }
        }
        info!(
            "Selected interpreter : {} ; with support level {:?}",
            name_best_interpreter, max_level_support
        );

        //launch !
        iter_types! {
            if Current::get_name() == name_best_interpreter {
                let mut inter = Current::new(self.data.clone());
                return inter.run();
            }
        }
        panic!()
    }
}
