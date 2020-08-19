use log::{info, LevelFilter};
use neovim_lib::{Neovim, NeovimApi, Session, Value};
use simple_logging::log_to_file;

use std::thread;

mod launcher;

mod interpreter;
mod interpreters;

#[derive(Debug)]
struct DataHolder {
    filetype: String,
    range: [i64; 2],
    filepath: String,
    projectroot: String,
    dependencies_path: Vec<String>,
}

impl DataHolder {
    fn new() -> Self {
        DataHolder {
            filetype: String::from(""),
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
                    let launcher = launcher::Launcher::new(self.data);
                    let mut i = launcher.select();
                    i.run();

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
