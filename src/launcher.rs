use crate::interpreter::InterpreterUtils;
use crate::*;
use error::SniprunError;
use interpreter::{Interpreter, SupportLevel};
use std::io::prelude::*;
use std::process::Command;
use std::{fs::File, io::Read};

pub struct Launcher {
    pub data: DataHolder,
}

impl Launcher {
    pub fn new(data: DataHolder) -> Self {
        Launcher { data }
    }

    fn match_filetype<T>(filetype: String, data: &DataHolder) -> bool
    where
        T: Interpreter,
    {
        if T::get_supported_languages().contains(&filetype) {
            return true;
        }

        if let Some(configured_filetypes) = T::get_interpreter_option(data, "use_on_filetypes") {
            if let Some(ft_array) = configured_filetypes.as_array() {
                return ft_array
                    .iter()
                    .map(|f| f.as_str().unwrap_or("####").to_owned())
                    .any(|f| f == filetype);
            }
        }

        false
    }

    pub fn select_and_run(&self) -> Result<String, SniprunError> {
        let selection = self.select();
        if let Some((name, level)) = selection {
            //launch !
            iter_types! {
                if Current::get_name() == name {
                    info!("[LAUNCHER] Selected interpreter: {}, at level {}", name, level);
                    let mut inter = Current::new_with_level(self.data.clone(), level);
                    return inter.run();
                }
            }
            info!("[LAUNCHER] Could not find a suitable interpreter");
            Err(SniprunError::CustomError(
                "could not find/run the selected interpreter".to_owned(),
            ))
        } else {
            Err(SniprunError::CustomError(String::from(
                "No filetype set for current file",
            )))
        }
    }

    pub fn select(&self) -> Option<(String, SupportLevel)> {
        if self.data.filetype.is_empty() {
            return None;
        }

        let mut max_level_support = SupportLevel::Unsupported;
        let mut name_best_interpreter = String::from("Generic");
        //select the best interpreter for the language
        let mut skip_all = false;
        iter_types! {
            if Launcher::match_filetype::<Current>(self.data.filetype.clone(), &self.data){
                if !skip_all && Current::get_max_support_level() > max_level_support {
                    max_level_support = Current::get_max_support_level();
                    name_best_interpreter = Current::get_name();
                }

                if self.data.selected_interpreters.contains(&Current::get_name()){
                    max_level_support = SupportLevel::Selected;
                    name_best_interpreter = Current::get_name();
                    skip_all = true;
                }

                if !skip_all && Current::default_for_filetype() {
                    max_level_support = Current::get_max_support_level();
                    name_best_interpreter = Current::get_name();
                    skip_all = true;
                }
            }
        }
        info!("selected {}", name_best_interpreter);
        let _ = skip_all; //silence false unused variable warning
        Some((name_best_interpreter, max_level_support))
    }

    pub fn info(&self) -> std::io::Result<String> {
        let mut v: Vec<String> = vec![];
        let filename = self.data.sniprun_root_dir.clone() + "/ressources/asciiart.txt";

        if let Ok(mut file) = File::open(filename) {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            info!("[INFO] Retrieved asciiart");
            v.push(content);
        } else {
            v.push(String::from("SNIPRUN"));
        }

        let gitscript = self.data.sniprun_root_dir.clone() + "/ressources/gitscript.sh";
        let mut get_version = Command::new(gitscript);
        get_version.current_dir(self.data.sniprun_root_dir.clone());
        if let Ok(res) = get_version.output() {
            info!("gitscript result: {:?}", res);
            if res.status.success() {
                let online_version = String::from_utf8(res.stdout).unwrap();
                info!("online version available: {}", &online_version);
                v.push(online_version);
            } else {
                v.push(String::from("Could not determine up-to-date status\n"));
            }
        } else {
            v.push(String::from("Could not determine up-to-date status\n"));
        }

        if let Some((name, level)) = self.select() {
            v.push(format!(
                "\nCurrently selected interpreter: {}, at support level: {}\n",
                name, level
            ));
            v.push(format!(
                "More information may be available via :SnipInfo {}\n\n",
                name
            ));
        } else {
            v.push("No interpreter selected\n\nYou can always get more info about one particular interpreter via:\n:SnipInfo <name>".to_string());
        }

        v.push("\nAvailable interpreters and languages".to_owned());

        let separator = "|----------------------|--------------|---------|-------------|------------|--------------|".to_string();
        v.push(separator.clone());
        v.push("| Interpreter          | Language     | Support | Default for |    REPL    | REPL enabled |".to_string());
        v.push("|                      |              |  Level  |  filetype   | capability |  by default  |".to_string());

        let mut temp_vec = vec![];
        iter_types! {
            let line = format!("| {:<21}| {:<13}| {:<8}|{:^13}|{:^12}|{:^14}|",
                    Current::get_name(),
                    Current::get_supported_languages().get(0).unwrap_or(&"".to_string()),
                    Current::get_max_support_level().to_string(),
                    match Current::default_for_filetype() {true => "yes" ,false => "no"},
                    match Current::has_repl_capability() { true => "yes" ,false => "no"},
                    match Current::behave_repl_like_default() { true => "yes" ,false => "no"},
                    // match Current::has_lsp_capability() { true => "yes" ,false => "no"}
                    );
            temp_vec.push(line);
        }

        temp_vec.sort();

        for (i, line) in temp_vec.iter().enumerate() {
            if i % 3 == 0 {
                v.push(separator.clone());
            }
            v.push(line.to_string());
        }

        v.push(separator);
        v.push("More help, quickstart and config options refresher can be found from: ':help sniprun'\n".to_owned());

        info!("[INFO] Writing info to file");
        let filename = self.data.work_dir.clone() + "/infofile.txt";
        let mut file = File::create(filename).unwrap();
        file.write_all(v.join("\n").as_bytes()).unwrap();
        Ok("".to_owned())
    }
}

#[cfg(test)]
mod test_launcher {

    use super::*;
    use std::env;

    #[test]
    fn run() {
        let mut data = DataHolder::new();
        data.filetype = String::from("pyt");
        data.current_line = String::from("println!(\"Hello\");");
        data.current_bloc = String::from("println!(\"Hello\");");
        data.range = [1, 1];

        let launcher = Launcher::new(data);
        let _res = launcher.select();
    }

    #[test]
    fn info() {
        let mut data = DataHolder::new();
        let path = env::current_dir().unwrap();
        data.sniprun_root_dir = path.display().to_string();

        data.filetype = String::from("rust");
        data.current_line = String::from("println!(\"Hello\");");
        data.current_bloc = String::from("println!(\"Hello\");");
        data.range = [1, 1];

        let launcher = Launcher::new(data);
        let _res = launcher.info().unwrap();
    }
}
