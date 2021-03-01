//! This is the documentation for the Sniprun project
//!
//! Sniprun is a neovim plugin that run parts of code.

use dirs::cache_dir;
use log::{info, LevelFilter};
use neovim_lib::{Neovim, NeovimApi, Session, Value};
use simple_logging::log_to_file;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

mod error;
mod interpreter;
mod interpreters;
mod launcher;

///This struct holds (with ownership) the data Sniprun and neovim
///give to the interpreter.
///This should be enough to implement up to project-level interpreters.
#[derive(Clone)]
pub struct DataHolder {
    /// contains the filetype of the file as return by `:set ft?`
    filetype: String,
    ///This contains the current line of code from where the user ran sniprun, and
    ///want to execute
    current_line: String,
    ///This contains the current block of text, if the user selected a bloc of code and ran snirpun
    ///on it
    current_bloc: String,

    ///The inclusive limits of the selected block (line numbers)
    range: [i64; 2],
    /// path of the current file that's being edited
    filepath: String,
    /// Field is left blank as of v0.3
    projectroot: String,
    /// field is left blank as of v0.3
    dependencies_path: Vec<String>,
    /// path to the cache directory that sniprun create
    work_dir: String,
    /// path to sniprun root, eg in case you need ressoruces from the ressources folder
    sniprun_root_dir: String,

    ///neovim instance
    nvim_instance: Option<Arc<Mutex<Neovim>>>,

    ///user config: selected interpreters
    selected_interpreters: Vec<String>,
    ///user config: repl behavior enabled list of interpreters
    repl_enabled: Vec<String>,
    ///user config: repl behavior disabled list of interpreters
    repl_disabled: Vec<String>,

    ///interpreter data
    interpreter_data: Option<Arc<Mutex<InterpreterData>>>,
}

#[derive(Clone, Default)]
///data that can be saved/accessed between Arc 2 interpreter runs
pub struct InterpreterData {
    ///indentifies the current interpreter (so that data from another interpreter does not get used
    owner: String,
    ///actual data, usually previous code selection for repl behavior
    content: String,

    /// PID of linked REPL if existing
    pid: Option<u32>,
}

impl DataHolder {
    ///create a new but almost empty DataHolder
    fn new() -> Self {
        std::fs::create_dir_all(format!(
            "{}/{}",
            cache_dir().unwrap().to_str().unwrap(),
            "sniprun"
        ))
        .unwrap();

        DataHolder {
            filetype: String::new(),
            current_line: String::new(),
            current_bloc: String::new(),
            range: [-1, -1],
            filepath: String::new(),
            projectroot: String::new(),
            dependencies_path: vec![],
            work_dir: format!("{}/{}", cache_dir().unwrap().to_str().unwrap(), "sniprun"),
            sniprun_root_dir: String::new(),
            nvim_instance: None,
            selected_interpreters: vec![],
            repl_enabled: vec![],
            repl_disabled: vec![],
            interpreter_data: None,
        }
    }
    ///remove and recreate the cache directory (is invoked by `:SnipReset`)
    fn clean_dir(&mut self) {
        let work_dir_path = self.work_dir.clone();
        std::fs::remove_dir_all(&work_dir_path).unwrap();
        std::fs::create_dir_all(&work_dir_path).unwrap();
    }
}

#[derive(Clone)]
struct EventHandler {
    nvim: Arc<Mutex<Neovim>>,
    data: DataHolder,
    interpreter_data: Arc<Mutex<InterpreterData>>,
}

enum Messages {
    Run,
    Clean,
    ClearReplMemory,
    Unknown(String),
}

impl From<String> for Messages {
    fn from(event: String) -> Self {
        match &event[..] {
            "run" => Messages::Run,
            "clean" => Messages::Clean,
            "clearrepl" => Messages::ClearReplMemory,
            _ => Messages::Unknown(event),
        }
    }
}

impl EventHandler {
    fn new() -> EventHandler {
        let session = Session::new_parent().unwrap();
        let nvim = Neovim::new(session);
        let mut data = DataHolder::new();
        let interpreter_data = Arc::new(Mutex::new(InterpreterData {
            owner: String::new(),
            content: String::new(),
            pid: None,
        }));
        data.interpreter_data = Some(interpreter_data.clone());

        EventHandler {
            nvim: Arc::new(Mutex::new(nvim)),
            data,
            interpreter_data: interpreter_data,
        }
    }

    /// fill the DataHolder with data from sniprun and Neovim
    fn fill_data(&mut self, values: Vec<Value>) {
        {
            info!("getting back eventual interpreter data");
            self.data.interpreter_data = Some(self.interpreter_data.clone());
        }

        {
            info!("filling data");
            self.data.range = [values[0].as_i64().unwrap(), values[1].as_i64().unwrap()];
            self.data.sniprun_root_dir = String::from(values[2].as_str().unwrap());
        }

        {
            //get filetype
            let ft = self.nvim.lock().unwrap().command_output("set ft?");
            if let Ok(real_ft) = ft {
                self.data.filetype = String::from(real_ft.split("=").last().unwrap());
            }
        }

        {
            //get current line
            let current_line = self.nvim.lock().unwrap().get_current_line();
            if let Ok(real_current_line) = current_line {
                self.data.current_line = real_current_line;
            }
            info!("got current_line");
        }

        {
            //get current bloc
            let mut nvim_instance = self.nvim.lock().unwrap();
            let current_bloc = nvim_instance.get_current_buf().unwrap().get_lines(
                &mut nvim_instance,
                self.data.range[0] - 1, //because the function is 0-based instead of 1 and end-exclusive
                self.data.range[1],
                false,
            );
            if let Ok(real_current_bloc) = current_bloc {
                self.data.current_bloc = real_current_bloc.join("\n");
            }
        }

        {
            //get full file path
            let full_file_path = self
                .nvim
                .lock()
                .unwrap()
                .command_output("echo expand('%:p')");
            if let Ok(real_full_file_path) = full_file_path {
                self.data.filepath = real_full_file_path;
            }
            info!("got filepath");
        }

        {
            //get nvim instance
            self.data.nvim_instance = Some(self.nvim.clone());
            info!("got nvim_instance");
        }
        {
            self.data.selected_interpreters = values[3]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_owned())
                .collect();
            info!("got selected interpreters");
        }
        {
            self.data.repl_enabled = values[4]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_owned())
                .collect();
            info!("got selected interpreters");
        }
        {
            self.data.repl_disabled = values[5]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_owned())
                .collect();
            info!("got selected interpreters");
        }
    }
}
enum HandleAction {
    New(thread::JoinHandle<()>),
}

fn main() {
    let mut event_handler = EventHandler::new();
    let _ = log_to_file(
        &format!("{}/{}", event_handler.data.work_dir, "sniprun.log"),
        LevelFilter::Info,
    );

    info!("[MAIN] SnipRun launched successfully");

    let receiver = event_handler
        .nvim
        .lock()
        .unwrap()
        .session
        .start_event_loop_channel();
    // let meh = Arc::new(Mutex::new(event_handler));

    let (send, recv) = mpsc::channel();
    thread::spawn(move || {
        let mut _handle: Option<thread::JoinHandle<()>> = None;
        loop {
            match recv.recv() {
                Err(_) => {
                    info!("Broken connection");
                    panic!("Broken connection")
                }
                Ok(HandleAction::New(new)) => _handle = Some(new),
            }
        }
    });

    //main loop
    info!("[MAIN] Start of main event loop");
    for (event, values) in receiver {
        match Messages::from(event.clone()) {
            //Run command
            Messages::Run => {
                info!("[MAINLOOP] Run command received");

                let mut event_handler2 = event_handler.clone();
                info!("[MAINLOOP] clone event handler");
                let _res2 = send.send(HandleAction::New(thread::spawn(move || {
                    // get up-to-date data
                    //
                    info!("[MAINLOOP] spawned thread");
                    event_handler2.fill_data(values);
                    info!("[MAINLOOP] filled dataholder");

                    //run the launcher (that selects, init and run an interpreter)
                    let launcher = launcher::Launcher::new(event_handler2.data.clone());
                    info!("[MAINLOOP] created launcher");
                    let result = launcher.select_and_run();
                    info!("[MAINLOOP] Interpreter return a result");

                    // return Ok(result) or Err(sniprunerror)
                    match result {
                        Ok(answer_str) => {
                            // do not display anything if string empty, as it may means the
                            // interpreter used the nvim handle directly and you don't want 
                            // to overwrite it!
                            if !answer_str.is_empty() {
                                //make sure there is no lone "
                                let mut answer_str = answer_str.clone();
                                answer_str = answer_str.replace("\\\"", "\"");
                                answer_str = answer_str.replace("\"", "\\\"");

                                //remove trailing /starting newlines
                                answer_str = answer_str.trim_start_matches('\n').trim_end_matches('\n').to_string();

                                info!("[MAINLOOP] Returning stdout of code run: {}", answer_str);

                                {
                                    let _ = event_handler2
                                        .nvim
                                        .lock()
                                        .unwrap()
                                        .command(&format!("echomsg \"{}\"", answer_str));
                                }
                            }
                        }
                        Err(e) => {
                            info!("[MAINLOOP] Returning an error");
                            let _ = event_handler2
                                .nvim
                                .lock()
                                .unwrap()
                                .err_writeln(&format!("{}", e));
                        }
                    };

                    //display ouput in nvim

                    //clean data
                    event_handler2.data = DataHolder::new();
                })));
            }
            Messages::Clean => {
                info!("[MAINLOOP] Clean command received");
                event_handler.data.clean_dir()
            }
            Messages::ClearReplMemory => {
                info!("[MAINLOOP] ClearReplMemory command received");
                event_handler.interpreter_data.lock().unwrap().owner.clear();
                event_handler
                    .interpreter_data
                    .lock()
                    .unwrap()
                    .content
                    .clear();
            }

            Messages::Unknown(event) => {
                info!("[MAINLOOP] Unknown event received: {:?}", event);
            }
        }
    }
}
