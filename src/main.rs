use log::{info, LevelFilter};
use neovim_lib::{Neovim, NeovimApi, Session};
use simple_logging::log_to_file;

struct DataHolder {
    filetype: String,
    range: [i64; 2],
    filepath: String,
    projectroot: String,
}

impl DataHolder {
    fn new() -> Self {
        DataHolder {
            filetype: String::from(""),
            range: [-1, -1],
            filepath: String::from(""),
            projectroot: String::from(""),
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
            match Messages::from(event) {
                //Run command
                Messages::Run => {
                    info!("run command received");
                    info!("trying line : {:?}", self.nvim.get_current_line());
                    info!(
                        "trying range : {:?}",
                        values
                            .iter()
                            .map(|v| v.as_i64().unwrap())
                            .collect::<Vec<i64>>()
                    );
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
}

fn main() {
    log_to_file("out.log", LevelFilter::Info);
    let mut event_handler = EventHandler::new();
    event_handler.recv();
}
