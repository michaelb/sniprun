//! This is the documentation for the Sniprun project
//!
//! Sniprun is a neovim plugin that run parts of code.

use dirs::cache_dir;
pub use display::{display, display_floating_window, DisplayFilter::*, DisplayType};
use log::{info, LevelFilter};
use neovim_lib::{Neovim, NeovimApi, Session, Value};
use simple_logging::log_to_file;
use std::str::FromStr;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub mod daemonizer;
pub mod display;
pub mod error;
pub mod interpreter;
pub mod interpreters;
pub mod launcher;

///This struct holds (with ownership) the data Sniprun and neovim
///give to the interpreter.
///This should be enough to implement up to project-level interpreters.
#[derive(Clone)]
pub struct DataHolder {
    /// contains the filetype of the file as return by `:set ft?`
    pub filetype: String,
    ///This contains the current line of code from where the user ran sniprun, and
    ///want to execute
    pub current_line: String,
    ///This contains the current block of text, if the user selected a bloc of code and ran snirpun
    ///on it
    pub current_bloc: String,

    ///The inclusive limits of the selected block (line numbers)
    pub range: [i64; 2],
    /// path of the current file that's being edited
    pub filepath: String,
    /// Current directory for Neovim. Neovim is responsible for setting correctly the current working directory
    /// Sniprun retrieves the result from `getcwd()` as of v1.2.9
    pub projectroot: String,
    /// field is left blank as of v0.3
    pub dependencies_path: Vec<String>,
    /// path to the cache directory that sniprun create
    pub work_dir: String,
    /// path to sniprun root, eg in case you need ressoruces from the ressources folder
    pub sniprun_root_dir: String,

    ///neovim instance
    pub nvim_instance: Option<Arc<Mutex<Neovim>>>,

    ///user config: selected interpreters
    pub selected_interpreters: Vec<String>,
    ///user config: repl behavior enabled list of interpreters
    pub repl_enabled: Vec<String>,
    ///user config: repl behavior disabled list of interpreters
    pub repl_disabled: Vec<String>,
    ///interpreter options
    pub interpreter_options: Option<Value>,

    ///interpreter data
    pub interpreter_data: Option<Arc<Mutex<InterpreterData>>>,

    /// whether to display echomsg-based messages (more compatibility)
    /// or new ones (multiline support) (default)
    pub return_message_type: ReturnMessageType,

    /// different way of displaying results
    pub display_type: Vec<DisplayType>,
    pub display_no_output: Vec<DisplayType>,

    pub cli_args: Vec<String>,

    pub nvim_pid: usize,
}

#[derive(Clone, Default, Debug)]
///data that can be saved/accessed between Arc 2 interpreter runs
pub struct InterpreterData {
    ///indentifies the current interpreter (so that data from another interpreter does not get used
    pub owner: String,
    ///actual data, usually previous code selection for repl behavior
    pub content: String,

    /// PID of linked REPL if existing
    pub pid: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReturnMessageType {
    EchoMsg,
    Multiline,
}

impl Default for DataHolder {
    fn default() -> Self {
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
            interpreter_options: None,
            interpreter_data: None,
            return_message_type: ReturnMessageType::Multiline,
            display_type: vec![DisplayType::Classic(Both)],
            display_no_output: vec![DisplayType::Classic(Both)],
            cli_args: vec![],
            nvim_pid: 0,
        }
    }
}

impl DataHolder {
    ///create a new but almost empty DataHolder
    pub fn new() -> Self {
        DataHolder::default()
    }
    ///remove and recreate the cache directory (is invoked by `:SnipReset`)
    pub fn clean_dir(&mut self) {
        let work_dir_path = self.work_dir.clone();
        std::fs::remove_dir_all(&work_dir_path).unwrap();
        std::fs::create_dir_all(&work_dir_path).unwrap();
    }

    pub fn modify_for_range(&mut self, range: (usize, usize)) {
        self.range = [range.0 as i64, range.1 as i64];
        let nvim_instance = self.nvim_instance.clone().unwrap();
        let mut nvim_instance = nvim_instance.lock().unwrap();
        let current_bloc = nvim_instance.get_current_buf().unwrap().get_lines(
            &mut nvim_instance,
            self.range[0] - 1,
            self.range[1],
            false,
        );
        if let Ok(real_current_bloc) = current_bloc {
            self.current_bloc = real_current_bloc.join("\n");
            self.current_line = real_current_bloc[0].to_string();
        }
    }
}

#[derive(Clone)]
pub struct EventHandler {
    nvim: Arc<Mutex<Neovim>>,
    data: DataHolder,
    pub interpreter_data: Arc<Mutex<InterpreterData>>,
}

enum Messages {
    Run,
    Clean,
    ClearReplMemory,
    Info,
    Ping,
    Unknown(String),
}

impl From<String> for Messages {
    fn from(event: String) -> Self {
        match &event[..] {
            "run" => Messages::Run,
            "clean" => Messages::Clean,
            "clearrepl" => Messages::ClearReplMemory,
            "ping" => Messages::Ping,
            "info" => Messages::Info,
            _ => Messages::Unknown(event),
        }
    }
}

impl Default for EventHandler {
    fn default() -> EventHandler {
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
            interpreter_data,
        }
    }
}

impl EventHandler {
    pub fn new() -> EventHandler {
        EventHandler::default()
    }

    fn index_from_name(&mut self, name: &str, config: &[(Value, Value)]) -> Option<usize> {
        for (i, kv) in config.iter().enumerate() {
            if name == kv.0.as_str().unwrap() {
                info!("looped on key {}", kv.0.as_str().unwrap());
                return Some(i);
            }
        }
        None
    }

    /// fill the DataHolder with data from sniprun and Neovim
    pub fn fill_data(&mut self, values: &[Value]) {
        // info!("[FILLDATA_ENTRY] received data from RPC: {:?}", values);
        let config = values[2].as_map().unwrap();
        {
            self.data.interpreter_data = Some(self.interpreter_data.clone());
            info!("[FILLDATA] got back eventual interpreter data");
        }

        info!("values length: {}", values.len());
        let cli_args = values[3].as_str().unwrap_or({
            info!("cli arguments are not a string: {:?}", values[3]);
            ""
        });
        {
            if !cli_args.is_empty() {
                self.data.cli_args = cli_args.split(' ').map(|s| s.to_string()).collect();
            }
        }

        {
            self.data.range = [values[0].as_i64().unwrap(), values[1].as_i64().unwrap()];
            info!("got data range: {:?}", self.data.range);
        }
        {
            if let Some(i) = self.index_from_name("sniprun_root_dir", config) {
                self.data.sniprun_root_dir = String::from(config[i].1.as_str().unwrap());
                info!("[FILLDATA] got sniprun root");
            }
        }

        {
            //get neovim's current directory
            let nvim_cwd = self
                .nvim
                .lock()
                .unwrap()
                .call_function("getcwd", vec![])
                .unwrap();
            info!("nvimcwd as value: nvim_cwd: {:?}", nvim_cwd);
            self.data.projectroot = String::from(nvim_cwd.as_str().unwrap());
            info!(
                "[FILLDATA] got neovim's current directory: {}",
                self.data.projectroot
            );
        }

        {
            //get filetype
            let ft = self.nvim.lock().unwrap().command_output("set ft?");
            if let Ok(real_ft) = ft {
                self.data.filetype = String::from(real_ft.split('=').last().unwrap());
            }
            info!("[FILLDATA] got filetype");
        }

        {
            //get current line
            let current_line = self.nvim.lock().unwrap().get_current_line();
            if let Ok(real_current_line) = current_line {
                self.data.current_line = real_current_line;
            }
            info!("[FILLDATA] got current_line");
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
            info!("[FILLDATA] got current_bloc");
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
            info!("[FILLDATA] got filepath");
        }

        {
            //get nvim instance
            self.data.nvim_instance = Some(self.nvim.clone());
            info!("[FILLDATA] got nvim_instance");
        }
        {
            if let Some(i) = self.index_from_name("selected_interpreters", config) {
                self.data.selected_interpreters = config[i]
                    .1
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect();
                info!("[FILLDATA] got selected interpreters");
            }
        }
        {
            if let Some(i) = self.index_from_name("repl_enable", config) {
                self.data.repl_enabled = config[i]
                    .1
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect();
                info!("[FILLDATA] got repl enabled interpreters");
            }
        }
        {
            if let Some(i) = self.index_from_name("repl_disable", config) {
                self.data.repl_disabled = config[i]
                    .1
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect();
                info!("[FILLDATA] got repl disabled interpreters");
            }
        }
        {
            if let Some(i) = self.index_from_name("display", config) {
                self.data.display_type = config[i]
                    .1
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap())
                    .map(DisplayType::from_str)
                    .inspect(|x| info!("[FILLDATA] display type found : {:?}", x))
                    .filter_map(|x| x.ok())
                    .collect();
                info!("[FILLDATA] got display types");
            }
        }
        {
            if let Some(i) = self.index_from_name("show_no_output", config) {
                self.data.display_no_output = config[i]
                    .1
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap())
                    .map(DisplayType::from_str)
                    .inspect(|x| {
                        info!("[FILLDATA] display type with 'no output'on found : {:?}", x)
                    })
                    .filter_map(|x| x.ok())
                    .collect();
                info!("[FILLDATA] got show_no_output");
            }
        }
        {
            if let Some(i) = self.index_from_name("inline_messages", config) {
                if config[i].1.as_i64().unwrap_or(0) == 1 {
                    self.data.return_message_type = ReturnMessageType::EchoMsg;
                } else {
                    self.data.return_message_type = ReturnMessageType::Multiline;
                }
                info!("[FILLDATA] got inline_messages setting");
            }
        }
        {
            if let Some(i) = self.index_from_name("neovim_pid", config) {
                if let Some(pid) = config[i].1.as_u64() {
                    self.data.nvim_pid = pid as usize;
                    info!("[FILLDATA] got neovim_pid value setting: {}", pid);
                } else {
                    info!("[FILLDATA] could get neovim_pid");
                }
            }
        }

        {
            self.data.interpreter_options = Some(values[2].clone());
        }

        info!("[FILLDATA] Done!");
    }

    pub fn override_data(&mut self, values: Vec<Value>) {
        if values.len() < 5 {
            info!("[OVERRIDE] No data to override");
            return;
        }
        if let Some(override_map) = values[4].as_map() {
            {
                if let Some(i) = self.index_from_name("filetype", override_map) {
                    if let Some(filetype_str) = override_map[i].1.as_str() {
                        if !filetype_str.is_empty() {
                            self.data.filetype = filetype_str.to_string();
                            info!("[OVERRIDE] filetype with: {}", filetype_str);
                        }
                    }
                }
            }
            {
                if let Some(i) = self.index_from_name("codestring", override_map) {
                    if let Some(codestring_str) = override_map[i].1.as_str() {
                        self.data.current_bloc = codestring_str.to_string();
                        self.data.current_line = codestring_str.to_string();
                        info!("[OVERRIDE] codestring with: {}", codestring_str);
                    }
                }
            }
        }
    }
}
enum HandleAction {
    New(thread::JoinHandle<()>),
}

pub fn start() {
    let mut event_handler = EventHandler::new();
    let _ = log_to_file(
        &format!("{}/{}", event_handler.data.work_dir, "sniprun.log"),
        LevelFilter::Info,
    );
    log_panics::init();

    info!("[MAIN] SnipRun launched successfully");

    let receiver = event_handler
        .nvim
        .lock()
        .unwrap()
        .session
        .start_event_loop_channel();

    let (send, recv) = mpsc::channel();
    thread::spawn(move || {
        let mut _handle: Option<thread::JoinHandle<()>> = None;
        loop {
            match recv.recv() {
                Err(_) => {
                    info!("[MAIN] Broken connection");
                    panic!("Broken connection")
                }
                Ok(HandleAction::New(new)) => _handle = Some(new),
            }
        }
    });

    //main loop
    info!("[MAIN] Start of main event loop");
    let mut counter = 1;
    for (event, values) in receiver {
        info!("EVENT {} RECEIVED ##################", counter);
        counter += 1;
        match Messages::from(event.clone()) {
            //Run command
            Messages::Run => {
                info!("[MAINLOOP] Run command received");

                let mut event_handler2 = event_handler.clone();
                info!("[RUN] clone event handler");
                let _ = send.send(HandleAction::New(thread::spawn(move || {
                    // get up-to-date data
                    //
                    info!("[RUN] spawned thread");
                    event_handler2.fill_data(&values);
                    event_handler2.override_data(values);
                    info!("[RUN] filled dataholder");

                    //run the launcher (that selects, init and run an interpreter)
                    let launcher = launcher::Launcher::new(event_handler2.data.clone());
                    info!("[RUN] created launcher");
                    let result = launcher.select_and_run();
                    info!("[RUN] Interpreter return a result");

                    if let Err(error::SniprunError::ReRunRanges(vec)) = result {
                        // sniprun separated into multiple ranges from
                        // markup languages code blocs
                        for range in vec {
                            let mut data = event_handler2.data.clone();
                            let nvim = data
                                .nvim_instance
                                .clone()
                                .expect("Nvim instance not available");
                            data.modify_for_range(range);
                            let launcher = launcher::Launcher::new(data.clone());
                            info!("[RUN] created launcher");
                            let result = launcher.select_and_run();
                            info!("[RUN] Interpreter return a result");
                            data.range[1] += 1; // display on end of code bloc
                            display(result, nvim, &data);
                        }
                    } else {
                        // normal, unique result
                        display(result, event_handler2.nvim, &event_handler2.data);
                    }

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
            Messages::Ping => {
                info!("[MAINLOOP] Ping command received");
            }

            Messages::Info => {
                info!("[MAINLOOP] Info command received");
                let mut event_handler2 = event_handler.clone();
                event_handler2.fill_data(&values);
                event_handler2.override_data(values);
                let launcher = launcher::Launcher::new(event_handler2.data.clone());
                let result = launcher.info();
                if let Ok(infomsg) = result {
                    display_floating_window(
                        &Ok(infomsg),
                        &event_handler2.nvim,
                        &event_handler.data,
                        false,
                        Both,
                    );
                }
            }

            Messages::Unknown(event) => {
                info!("[MAINLOOP] Unknown event received: {:?}", event);
            }
        }
    }
}

#[cfg(test)]
mod test_main {
    use super::*;

    #[test]
    fn test_main() {
        let mut event_handler = fake_event();
        let _ = log_to_file(&"test_sniprun.log".to_string(), LevelFilter::Info);

        event_handler.fill_data(&fake_msgpack());
        event_handler.data.filetype = String::from("javascript");
        event_handler.data.current_bloc = String::from("console.log(\"Hello, World!\")");
        //run the launcher (that selects, init and run an interpreter)
        let launcher = launcher::Launcher::new(event_handler.data.clone());
        info!("[RUN] created launcher");
        let result = launcher.select_and_run();
        info!("[RUN] Interpreter return a result");

        display(result, event_handler.nvim, &event_handler.data);
    }

    pub fn fake_event() -> EventHandler {
        let session = Session::new_child().unwrap();
        let mut nvim = Neovim::new(session);
        let mut data = DataHolder::new();
        let interpreter_data = Arc::new(Mutex::new(InterpreterData {
            owner: String::new(),
            content: String::new(),
            pid: None,
        }));
        let _receiver = nvim.session.start_event_loop_channel();
        data.interpreter_data = Some(interpreter_data.clone());
        EventHandler {
            nvim: Arc::new(Mutex::new(nvim)),
            data,
            interpreter_data,
        }
    }

    pub fn fake_msgpack() -> Vec<Value> {
        let mut data: Vec<Value> = Vec::new();

        let line_start = Value::from(1);
        let line_end = Value::from(2);
        data.push(line_start);
        data.push(line_end);

        let mut config_as_vec: Vec<(Value, Value)> = vec![
            (
                Value::from("selected_interpreters"),
                Value::from(Vec::<Value>::new()),
            ),
            (Value::from("repl_enable"), Value::from(Vec::<Value>::new())),
            (
                Value::from("repl_disable"),
                Value::from(Vec::<Value>::new()),
            ),
        ];

        let display_types: Vec<Value> = vec![
            Value::from("Classic"),
            Value::from("Terminal"),
            Value::from("TerminalWithCode"),
            Value::from("VirtualTextOk"),
            Value::from("VirtualTextErr"),
            Value::from("TempFloatingWindow"),
        ];

        config_as_vec.push((Value::from("display"), Value::from(display_types)));
        config_as_vec.push((
            Value::from("sniprun_root_dir"),
            Value::from("/tmp/notimportant"),
        ));
        config_as_vec.push((Value::from("inline_messages"), Value::from(0)));

        data.push(Value::from(config_as_vec));
        data.push(Value::from(""));

        data
    }
}
