use log::{info, LevelFilter};
use neovim_lib::{Neovim, NeovimApi, Session, Value};
use simple_logging::log_to_file;

use std::thread;

mod error;
mod interpreter;
mod interpreters;
mod launcher;

use interpreter::Interpreter;

#[derive(Debug, Clone, PartialEq)]
pub struct DataHolder {
    filetype: String,
    current_line: String,
    current_bloc: String,
    range: [i64; 2],
    filepath: String,
    projectroot: String,
    dependencies_path: Vec<String>,
}

impl DataHolder {
    fn new() -> Self {
        DataHolder {
            filetype: String::from(""),
            current_line: String::from(""),
            current_bloc: String::from(""),
            range: [-1, -1],
            filepath: String::from(""),
            projectroot: String::from(""),
            dependencies_path: vec![],
        }
    }
}

struct EventHandler {
    nvim: Neovim,
    data: DataHolder,
}

enum Messages {
    Run,
    Terminate,
    Unknown(String),
}

impl From<String> for Messages {
    fn from(event: String) -> Self {
        match &event[..] {
            "run" => Messages::Run,
            "terminate" => Messages::Terminate,
            _ => Messages::Unknown(event),
        }
    }
}

impl EventHandler {
    fn new() -> EventHandler {
        let session = Session::new_parent().unwrap();
        let nvim = Neovim::new(session);
        let data = DataHolder::new();
        EventHandler {
            nvim: nvim,
            data: data,
        }
    }

    fn recv(&mut self) {
        let receiver = self.nvim.session.start_event_loop_channel();

        for (event, values) in receiver {
            info!("inside loop: {:?}", event);
            match Messages::from(event.clone()) {
                //Run command
                Messages::Run => {
                    info!("run command received");
                    self.fill_data(&event, values);
                    //run the interpreter
                    let launcher = launcher::Launcher::new(self.data.clone());
                    let answer = match launcher.select_and_run() {
                        Ok(answer_str) => answer_str,
                        Err(e) => format!("{}", e),
                    };
                    info!("answer: {}", answer);

                    //display ouput in nvim
                    let res = self.nvim.command(&format!("echo \"{}\"", answer));
                    info!("echoing back results : {:?}", res);

                    //clean data
                    self.data = DataHolder::new();
                }

                Messages::Terminate => {
                    // self.nvim.command(&format!("echo terminating")).unwrap();
                    info!("terminate command received");
                }

                Messages::Unknown(event) => {
                    info!("unknown event received: {:?}", event);
                }
            }
        }
    }

    fn fill_data(&mut self, event: &str, values: Vec<Value>) {
        self.data.range = [values[0].as_i64().unwrap(), values[1].as_i64().unwrap()];

        //get filetype
        let ft = self.nvim.command_output("set ft?");
        if let Ok(real_ft) = ft {
            self.data.filetype = String::from(real_ft.split("=").last().unwrap());
        }

        //get current line
        let current_line = self.nvim.get_current_line();
        if let Ok(real_current_line) = current_line {
            self.data.current_line = real_current_line;
        }

        //get current bloc
        let current_bloc = self.nvim.get_current_buf().unwrap().get_lines(
            &mut self.nvim,
            self.data.range[0] - 1, //because the function is 0-based instead of 1 and end-exclusive
            self.data.range[1],
            false,
        );
        if let Ok(real_current_bloc) = current_bloc {
            self.data.current_bloc = real_current_bloc.join("\n");
        }

        //get full file path
        let full_file_path = self.nvim.command_output("echo expand('%:p')");
        if let Ok(real_full_file_path) = full_file_path {
            self.data.filepath = real_full_file_path;
        }

        info!("data : {:?}", self.data);
    }
}

fn main() {
    log_to_file("out.log", LevelFilter::Info);
    let mut event_handler = EventHandler::new();
    event_handler.recv();
}
