use crate::*;
use interpreter::Interpreter;

pub struct Launcher {
    pub data: DataHolder,
}

impl Launcher {
    pub fn new(data: DataHolder) -> Self {
        Launcher { data: data }
    }

    pub fn select_and_run<'a>(&self) -> Result<String, String> {
        iter_types! {
            if Current::get_supported_languages().contains(&self.data.filetype) {
                // later, check and sort for best support level
                let mut inter = Current::new(self.data.clone());
                return inter.run();
            }
        }
        panic!()
    }
}
