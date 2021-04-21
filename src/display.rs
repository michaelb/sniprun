use crate::error::SniprunError;
use crate::{DataHolder, ReturnMessageType};
use log::info;
use neovim_lib::{Neovim, NeovimApi};
use std::fmt;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Ord, PartialOrd, PartialEq, Eq)]
pub enum DisplayType {
    Classic = 0,
    VirtualTextOk,
    VirtualTextErr,
    Terminal,
    TempFloatingWindow,
}
use DisplayType::*;

impl FromStr for DisplayType {
    type Err = SniprunError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Classic" => Ok(Classic),
            "VirtualTextOk" => Ok(VirtualTextOk),
            "VirtualTextErr" => Ok(VirtualTextErr),
            "Terminal" => Ok(Terminal),
            "TempFloatingWindow" => Ok(TempFloatingWindow),
            _ => Err(SniprunError::InternalError(
                "Invalid display type: ".to_string() + &s,
            )),
        }
    }
}

impl fmt::Display for DisplayType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match &self {
            DisplayType::Classic => "Classic",
            DisplayType::VirtualTextOk => "VirtualTextOk",
            DisplayType::VirtualTextErr => "VirtualTextErr",
            DisplayType::Terminal => "Terminal",
            DisplayType::TempFloatingWindow => "TempFloatingWindow",
        };
        write!(f, "{}", name)
    }
}

pub fn display(result: Result<String, SniprunError>, nvim: Arc<Mutex<Neovim>>, data: &DataHolder) {
    let mut display_type = data.display_type.clone();
    display_type.sort();
    display_type.dedup(); //now only uniques display types
    info!("Display type chosen: {:?}", display_type);

    for dt in display_type.iter() {
        match dt {
            Classic => return_message_classic(&result, &nvim, &data.return_message_type),
            VirtualTextOk => display_virtual_text(&result, &nvim, &data, true),
            VirtualTextErr => display_virtual_text(&result, &nvim, &data, false),
            Terminal => display_terminal(),
            TempFloatingWindow => display_temp_floating_window(&result, &nvim, &data),
        }
    }
}

pub fn display_virtual_text(
    result: &Result<String, SniprunError>,
    nvim: &Arc<Mutex<Neovim>>,
    data: &DataHolder,
    is_ok: bool,
) {
    if is_ok != result.is_ok() {
        return; //don't display unasked-for things
    }

    let namespace_id = nvim.lock().unwrap().create_namespace("sniprun").unwrap();

    let last_line = data.range[1] - 1;
    let res = nvim.lock().unwrap().command(&format!(
        "call nvim_buf_clear_namespace(0,{},{},{})",
        namespace_id,
        last_line,
        last_line + 1
    ));
    info!("cleared previous virtual_text? {:?}", res);

    let hl_ok = "SniprunVirtualTextOk";
    let hl_err = "SniprunVirtualTextErr";
    let res = match result {
        Ok(message_ok) => nvim.lock().unwrap().command(&format!(
            "call nvim_buf_set_virtual_text(0,{},{},[[\"{}\",\"{}\"]], [])",
            namespace_id,
            last_line,
            shorten_ok(&cleanup_and_escape(message_ok)),
            hl_ok
        )),
        Err(message_err) => nvim.lock().unwrap().command(&format!(
            "call nvim_buf_set_virtual_text(0,{},{},[[\"{}\",\"{}\"]], [])",
            namespace_id,
            last_line,
            shorten_err(&cleanup_and_escape(&message_err.to_string())),
            hl_err
        )),
    };
    info!("done displaying virtual text, {:?}", res);
}

fn shorten_ok(message: &str) -> String {
    let marker = "<- ";
    marker.to_string()
        + message
            .lines()
            .filter(|&s| !s.is_empty())
            .last()
            .unwrap_or("()")
}

fn shorten_err(message: &str) -> String {
    let marker = "<- ";
    marker.to_string() + message.lines().next().unwrap_or("()")
}

fn cleanup_and_escape(message: &str) -> String {
    let answer_str = message.replace("\\", "\\\\");
    let answer_str = answer_str.replace("\\\"", "\"");
    let answer_str = answer_str.replace("\"", "\\\"");

    //remove trailing /starting newlines
    let answer_str = answer_str
        .trim_start_matches('\n')
        .trim_end_matches('\n')
        .to_string();
    let answer_str = answer_str.replace("\n", "\\\n");
    answer_str
}

pub fn display_terminal() {}

pub fn display_temp_floating_window(
    message: &Result<String, SniprunError>,
    nvim: &Arc<Mutex<Neovim>>,
    data: &DataHolder,
) {
    let col = data
        .current_bloc
        .lines()
        .last()
        .unwrap_or(&data.current_line)
        .len();
    let row = data.range[1] - 1;
    info!(
        "trying to open a floating window on row, col = {}, {}",
        row, col
    );
    let mes = message.as_ref().unwrap();
    info!(
        "message = {}",
        &format!(
            "lua require\"sniprun.display\".fw_open({},{},\"{}\", true)",
            row,
            col,
            cleanup_and_escape(&mes)
        )
    );

    let _res = match message {
        Ok(result) => nvim.lock().unwrap().command(&format!(
            "lua require\"sniprun.display\".fw_open({},{},\"{}\", true)",
            row,
            col,
            cleanup_and_escape(&result)
        )),
        Err(result) => nvim.lock().unwrap().command(&format!(
            "lua require\"sniprun.display\".fw_open({},{},\"{}\", false)",
            row,
            col,
            cleanup_and_escape(&result.to_string())
        )),
    };
    info!("res = {:?}", _res);
}

pub fn return_message_classic(
    message: &Result<String, SniprunError>,
    nvim: &Arc<Mutex<Neovim>>,
    rmt: &ReturnMessageType,
) {
    match message {
        Ok(answer_ok) => {
            //make sure there is no lone "
            let mut answer_str = cleanup_and_escape(answer_ok);
            info!("Final str {}", answer_str);

            match rmt {
                ReturnMessageType::Multiline => {
                    let _ = nvim
                        .lock()
                        .unwrap()
                        .command(&format!("echo \"{}\"", answer_str));
                }
                ReturnMessageType::EchoMsg => {
                    let _ = nvim
                        .lock()
                        .unwrap()
                        .command(&format!("echomsg \"{}\"", answer_str));
                }
            }
        }
        Err(e) => match rmt {
            ReturnMessageType::Multiline => {
                let _ = nvim.lock().unwrap().err_writeln(&format!("{}", e));
            }
            ReturnMessageType::EchoMsg => {
                let _ = nvim.lock().unwrap().command(&format!(
                    "echohl ErrorMsg | echomsg \"{}\" | echohl None",
                    e
                ));
            }
        },
    }
}
