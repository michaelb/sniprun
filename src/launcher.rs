use crate::*;
use error::SniprunError;
use interpreter::{Interpreter, SupportLevel};
use std::{fs::File, io::Read};
use std::process::Command;

pub struct Launcher {
    pub data: DataHolder,
}

impl Launcher {
    pub fn new(data: DataHolder) -> Self {
        Launcher { data }
    }

    pub fn select_and_run<'a>(&self) -> Result<String, SniprunError> {
        if self.data.filetype.is_empty() {
            return Err(SniprunError::CustomError(String::from(
                "No filetype set for current file",
            )));
        }

        let mut max_level_support = SupportLevel::Unsupported;
        let mut name_best_interpreter = String::from("Generic");
        //select the best interpreter for the language
        let mut skip_all = false;
        iter_types! {
            if !skip_all && Current::get_supported_languages().contains(&self.data.filetype){
                if Current::get_max_support_level() > max_level_support {
                    max_level_support = Current::get_max_support_level();
                    name_best_interpreter = Current::get_name();
                }

                if self.data.selected_interpreters.contains(&Current::get_name()){
                    max_level_support = SupportLevel::Selected;
                    name_best_interpreter = Current::get_name();
                    skip_all = true;
                }

                if Current::default_for_filetype() {
                    max_level_support = Current::get_max_support_level();
                    name_best_interpreter = Current::get_name();
                    skip_all = true;
                }
            }
        }
        let _ = skip_all; //silence false unused variable warning
        info!(
            "[LAUNCHER] Selected interpreter : {} ; with support level {:?}",
            name_best_interpreter, max_level_support
        );

        //launch !
        iter_types! {
            if Current::get_name() == name_best_interpreter {
                let mut inter = Current::new_with_level(self.data.clone(), max_level_support);
                return inter.run();
            }
        }
        panic!()
    }

    pub fn info(&self) -> std::io::Result<String>{
        let mut v : Vec<String> = vec![];
        let filename = self.data.sniprun_root_dir.clone() + "/ressources/asciiart.txt";
        let mut file  = File::open(filename)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        info!("Retrieved asciiart");
        v.push(content);
        v.push("\n".to_string());


        
        let gitscript = self.data.sniprun_root_dir.clone() + "/ressources/gitscript.sh";
        let mut get_version = Command::new(gitscript);
        get_version.current_dir(self.data.sniprun_root_dir.clone());
        let res = get_version.output().expect("process failed to execute");
        if res.status.success(){
            let online_version = String::from_utf8(res.stdout).unwrap();
            info!("online version available: {}", online_version);
            v.push(online_version);
            v.push("".to_string());
        }


        v.push("|--------------------------|--------------|-------------|------------|--------------|------------|".to_string());
        v.push("| Interpreter              | Language     | Default for |    REPL    | REPL enabled | Treesitter |".to_string());
        v.push("|                          |              |  filetype   | capability |  by default  | capability |".to_string());
        v.push("|--------------------------|--------------|-------------|------------|--------------|------------|".to_string());


        iter_types! {
            let line = String::new() + 
                &format!("| {:<25}| {:<13}|{:^13}|{:^12}|{:^14}|{:^11}|",
                    Current::get_name(), 
                    Current::get_supported_languages().iter().next().unwrap_or(&"".to_string()),
                    match Current::default_for_filetype() {true => "✔" ,false => "✘"},
                    match Current::behave_repl_like_default() { true => "✔" ,false => "✘"},
                    match Current::has_repl_capability() { true => "✔" ,false => "✘"},
                    match Current::has_treesitter_capability() { true => "✔" ,false => "✘"}
                    );
            v.push(line);
        }
        v.push("|--------------------------|--------------|-------------|------------|--------------|------------|\n".to_string());

        Ok(v.join("\n"))
    }
}
