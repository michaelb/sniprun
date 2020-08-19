use crate::*;
use interpreter::Interpreter;

pub struct Launcher {
    data: DataHolder,
}

impl Launcher {
    pub fn new(data: DataHolder) -> Self {
        Launcher { data: data }
    }

    pub fn select(&self) -> impl Interpreter {
        iter_types! {
            if Current::get_supported_languages().contains(self.data.filetype) {
                // later, check and sort for best support level
                let inter = Current::new(self.data);
                return inter;
            }
        }
        panic!()
    }
}
