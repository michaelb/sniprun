use crate::error::SniprunError;
use crate::{DataHolder, ReturnMessageType};
use log::info;
use neovim_lib::{Neovim, NeovimApi};
use std::fmt;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use unindent::Unindent;

#[derive(Clone, Debug, Ord, PartialOrd, PartialEq, Eq)]
pub enum DisplayType {
    Classic = 0,
    NvimNotify,
    VirtualTextOk,
    VirtualTextErr,
    Terminal,
    TerminalWithCode,
    LongTempFloatingWindow,
    TempFloatingWindow,
    Api,
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
            "TerminalWithCode" => Ok(TerminalWithCode),
            "LongTempFloatingWindow" => Ok(LongTempFloatingWindow),
            "TempFloatingWindow" => Ok(TempFloatingWindow),
            "Api" => Ok(Api),
            "NvimNotify" => Ok(NvimNotify),
            _ => Err(SniprunError::InternalError(
                "Invalid display type: ".to_string() + s,
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
            DisplayType::TerminalWithCode => "TerminalWithCode",
            DisplayType::LongTempFloatingWindow => "LongTempFloatingWindow",
            DisplayType::TempFloatingWindow => "TempFloatingWindow",
            DisplayType::Api => "Api",
            DisplayType::NvimNotify => "NvimNotify",
        };
        write!(f, "{}", name)
    }
}

pub fn display(result: Result<String, SniprunError>, nvim: Arc<Mutex<Neovim>>, data: &DataHolder) {
    let mut display_type = data.display_type.clone();
    display_type.sort();
    display_type.dedup(); //now only uniques display types

    // remove transparently incompatible/redundant displays
    if display_type.contains(&TerminalWithCode) {
        display_type.retain(|dt| dt != &Terminal);
    }

    info!("Display type chosen: {:?}", display_type);
    for dt in display_type.iter() {
        match dt {
            Classic => return_message_classic(&result, &nvim, &data.return_message_type, data),
            VirtualTextOk => display_virtual_text(&result, &nvim, data, true),
            VirtualTextErr => display_virtual_text(&result, &nvim, data, false),
            Terminal => display_terminal(&result, &nvim, data),
            TerminalWithCode => display_terminal_with_code(&result, &nvim, data),
            LongTempFloatingWindow => display_floating_window(&result, &nvim, data, true),
            TempFloatingWindow => display_floating_window(&result, &nvim, data, false),
            Api => send_api(&result, &nvim, data),
            NvimNotify => display_nvim_notify(&result, &nvim, data),
        }
    }
}

pub fn display_nvim_notify(
    message: &Result<String, SniprunError>,
    nvim: &Arc<Mutex<Neovim>>,
    data: &DataHolder,
) {
    let res = match message {
        Ok(result) => nvim.lock().unwrap().command(&format!(
            "lua require\"sniprun.display\".display_nvim_notify(\"{}\", true)",
            no_output_wrap(result, data, &DisplayType::NvimNotify).replace("\n", "\\\n"),
        )),
        Err(result) => nvim.lock().unwrap().command(&format!(
            "lua require\"sniprun.display\".display_nvim_notify(\"{}\", false)",
            no_output_wrap(&result.to_string(), data, &DisplayType::NvimNotify)
                .replace("\n", "\\\n"),
        )),
    };
    info!("display notify res = {:?}", res);
}

pub fn send_api(
    message: &Result<String, SniprunError>,
    nvim: &Arc<Mutex<Neovim>>,
    data: &DataHolder,
) {
    let res = match message {
        Ok(result) => {
            let mut nvim_instance = nvim.lock().unwrap();
            nvim_instance.command(&format!(
                "lua require\"sniprun.display\".send_api(\"{}\", true)",
                no_output_wrap(result, data, &DisplayType::Api).replace("\n", "\\\n"),
            ))
        }
        Err(result) => {
            let mut nvim_instance = nvim.lock().unwrap();
            nvim_instance.command(&format!(
                "lua require\"sniprun.display\".send_api(\"{}\", false)",
                no_output_wrap(&result.to_string(), data, &DisplayType::Api).replace("\n", "\\\n"),
            ))
        }
    };
    if res.is_ok() {
        info!("done displaying api");
    } else {
        info!("failed to display via api");
    }
}

pub fn display_virtual_text(
    result: &Result<String, SniprunError>,
    nvim: &Arc<Mutex<Neovim>>,
    data: &DataHolder,
    is_ok: bool,
) {
    info!("range is : {:?}", data.range);
    let namespace_id = nvim.lock().unwrap().create_namespace("sniprun").unwrap();
    if is_ok != result.is_ok() {
        if let Err(SniprunError::InterpreterLimitationError(_)) = result {
            return; // without clearing the line
        }
        // clear the current line
        let last_line = data.range[1] - 1;
        let _ = nvim.lock().unwrap().command(&format!(
            "call nvim_buf_clear_namespace(0,{},{},{})",
            namespace_id,
            data.range[0] - 1,
            last_line + 1
        ));

        return; //don't display unasked-for things
    }

    info!("namespace_id = {:?}", namespace_id);

    let last_line = data.range[1] - 1;
    let res = nvim.lock().unwrap().command(&format!(
        "call nvim_buf_clear_namespace(0,{},{},{})",
        namespace_id,
        data.range[0] - 1,
        last_line + 1
    ));
    info!("cleared previous virtual_text? {:?}", res);

    let hl_ok = "SniprunVirtualTextOk";
    let hl_err = "SniprunVirtualTextErr";
    let res = match result {
        Ok(message_ok) => {
            if shorten_ok(&no_output_wrap(
                message_ok,
                data,
                &DisplayType::VirtualTextOk,
            ))
            .is_empty()
            {
                return;
            }
            nvim.lock().unwrap().command(&format!(
                "lua require\"sniprun.display\".display_extmark({},{},\"{}\",\"{}\")",
                namespace_id,
                last_line,
                shorten_ok(&no_output_wrap(
                    message_ok,
                    data,
                    &DisplayType::VirtualTextOk
                )),
                hl_ok
            ))
        }
        Err(message_err) => {
            if shorten_err(&no_output_wrap(
                &message_err.to_string(),
                data,
                &DisplayType::VirtualTextErr,
            ))
            .is_empty()
            {
                return;
            }
            nvim.lock().unwrap().command(&format!(
                "lua require\"sniprun.display\".display_extmark({},{},\"{}\",\"{}\")",
                namespace_id,
                last_line,
                shorten_err(&no_output_wrap(
                    &message_err.to_string(),
                    data,
                    &DisplayType::VirtualTextErr
                )),
                hl_err
            ))
        }
    };
    info!("done displaying virtual text, {:?}", res);
}

pub fn display_terminal(
    message: &Result<String, SniprunError>,
    nvim: &Arc<Mutex<Neovim>>,
    data: &DataHolder,
) {
    info!("data_bloc = {}", data.current_bloc);
    let a = data.current_bloc.lines();
    info!("length = {}", a.count());
    let res = match message {
        Ok(result) => nvim.lock().unwrap().command(&format!(
            "lua require\"sniprun.display\".write_to_term(\"{}\", true)",
            no_output_wrap(result, data, &DisplayType::Terminal).replace("\n", "\\\n"),
        )),
        Err(result) => nvim.lock().unwrap().command(&format!(
            "lua require\"sniprun.display\".write_to_term(\"{}\", false)",
            no_output_wrap(&result.to_string(), data, &DisplayType::Terminal).replace("\n", "\\\n"),
        )),
    };
    info!("display terminal res = {:?}", res);
}

pub fn display_terminal_with_code(
    message: &Result<String, SniprunError>,
    nvim: &Arc<Mutex<Neovim>>,
    data: &DataHolder,
) {
    let res = match message {
        Ok(result) => nvim.lock().unwrap().command(&format!(
            "lua require\"sniprun.display\".write_to_term(\"{}\\n{}\", true)",
            cleanup_and_escape(
                &format!("\n{}", &data.current_bloc)
                    .unindent()
                    .lines()
                    .fold("".to_string(), |cur_bloc, line_in_bloc| {
                        cur_bloc + "> " + line_in_bloc + "\n"
                    })
            )
            .replace("\n", "\\\n"),
            no_output_wrap(result, data, &DisplayType::TerminalWithCode).replace("\n", "\\\n"),
        )),
        Err(result) => nvim.lock().unwrap().command(&format!(
            "lua require\"sniprun.display\".write_to_term(\"{}\\n{}\", false)",
            cleanup_and_escape(
                &format!("\n{}", &data.current_bloc)
                    .unindent()
                    .lines()
                    .fold("".to_string(), |cur_bloc, line_in_bloc| {
                        cur_bloc + "> " + line_in_bloc + "\n"
                    })
            )
            .replace("\n", "\\\n"),
            no_output_wrap(&result.to_string(), data, &DisplayType::TerminalWithCode)
                .replace("\n", "\\\n"),
        )),
    };
    info!("display terminal res = {:?}", res);
}

pub fn display_floating_window(
    message: &Result<String, SniprunError>,
    nvim: &Arc<Mutex<Neovim>>,
    data: &DataHolder,
    long_only: bool,
) {
    if long_only {
        let do_no_display = match message {
            Ok(message_ok) => message_ok.lines().count() <= 1,
            Err(message_err) => message_err.to_string().lines().count() <= 1,
        };
        if do_no_display {
            return; //do not display short messages
        }
    }

    let col = data
        .current_bloc
        .lines()
        .filter(|&line| !line.is_empty())
        .last()
        .unwrap_or(&data.current_line)
        .len();
    let row = data.range[0] + data.current_bloc.trim_end_matches('\n').lines().count() as i64 - 1;
    info!(
        "trying to open a floating window on row, col = {}, {}",
        row, col
    );

    let res = match message {
        Ok(result) => nvim.lock().unwrap().command(&format!(
            "lua require\"sniprun.display\".fw_open({},{},\"{}\", true)",
            row - 1,
            col,
            no_output_wrap(&result.to_string(), data, &DisplayType::TempFloatingWindow)
                .replace("\n", "\\\n"),
        )),
        Err(result) => nvim.lock().unwrap().command(&format!(
            "lua require\"sniprun.display\".fw_open({},{},\"{}\", false)",
            row - 1,
            col,
            no_output_wrap(&result.to_string(), data, &DisplayType::TempFloatingWindow)
                .replace("\n", "\\\n"),
        )),
    };
    info!("display floating window res = {:?}", res);
}

pub fn return_message_classic(
    message: &Result<String, SniprunError>,
    nvim: &Arc<Mutex<Neovim>>,
    rmt: &ReturnMessageType,
    data: &DataHolder,
) {
    match message {
        Ok(answer_ok) => {
            //make sure there is no lone "
            let answer_str = no_output_wrap(answer_ok, data, &DisplayType::Classic);
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

fn shorten_ok(message: &str) -> String {
    if message.is_empty() {
        return String::new();
    }

    let mut marker = String::from("<- ");
    if message.lines().count() > 1 {
        marker += &".".repeat(std::cmp::max(2, std::cmp::min(6, message.lines().count())));
    }

    marker.to_string()
        + message
            .lines()
            .filter(|&s| !s.is_empty())
            .last()
            .unwrap_or("")
}

fn shorten_err(message: &str) -> String {
    if message.is_empty() {
        return String::new();
    }
    let mut marker = String::from("<- ") + message.lines().next().unwrap_or("");
    if message.lines().count() > 1 {
        marker += &".".repeat(std::cmp::max(3, std::cmp::min(10, message.lines().count())));
    }
    marker
}

fn cleanup_and_escape(message: &str) -> String {
    let mut escaped = String::with_capacity(message.len());
    for c in message.chars() {
        match c {
            '\x08' => escaped += "\\b",
            '\x0c' => escaped += "\\f",
            '\t' => escaped += "\\t",
            '"' => escaped += "\\\"",
            '\\' => escaped += "\\\\",
            c => escaped += &c.to_string(),
        }
    }

    //remove trailing /starting newlines
    let answer_str = escaped
        .trim_start_matches('\n')
        .trim_end_matches('\n')
        .to_string();
    answer_str
}

fn no_output_wrap(message: &str, data: &DataHolder, current_type: &DisplayType) -> String {
    let message_clean = cleanup_and_escape(message);
    for dt in data.display_no_output.iter() {
        if dt == current_type && message_clean.is_empty() {
            info!("Empty message converted to 'no output')");
            return String::from("(no output)");
        }
    }
    info!("message '{}' cleaned out", message_clean);
    message_clean
}
