use crate::error::SniprunError;
use crate::{DataHolder, ReturnMessageType};
use log::info;
use neovim_lib::{Neovim, NeovimApi};
use std::fmt;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use unindent::Unindent;

#[derive(Clone, Copy, Debug, Ord, PartialOrd)]
pub enum DisplayFilter {
    OnlyOk,
    OnlyErr,
    Both,
}

// abusing a little the system
impl PartialEq for DisplayFilter {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Both => true,
            OnlyOk => !matches!(other, OnlyErr),
            OnlyErr => !matches!(other, OnlyOk),
        }
    }
}
impl Eq for DisplayFilter {}

use DisplayFilter::*;

#[derive(Clone, Debug, Ord, PartialOrd, PartialEq, Eq)]
pub enum DisplayType {
    Classic(DisplayFilter),
    NvimNotify(DisplayFilter),
    VirtualText(DisplayFilter),
    Terminal(DisplayFilter),
    TerminalWithCode(DisplayFilter),
    LongTempFloatingWindow(DisplayFilter),
    TempFloatingWindow(DisplayFilter),
    Api(DisplayFilter),
}
use DisplayType::*;

impl FromStr for DisplayType {
    type Err = SniprunError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let display_filter;
        let mut display_type = s;
        if s.ends_with("Ok") {
            display_filter = OnlyOk;
            display_type = s.strip_suffix("Ok").unwrap();
        } else if s.ends_with("Err") {
            display_filter = OnlyErr;
            display_type = s.strip_suffix("Err").unwrap();
        } else {
            display_filter = Both;
        }
        match display_type {
            "Classic" => Ok(Classic(display_filter)),
            "VirtualText" => Ok(VirtualText(display_filter)),
            "Terminal" => Ok(Terminal(display_filter)),
            "TerminalWithCode" => Ok(TerminalWithCode(display_filter)),
            "LongTempFloatingWindow" => Ok(LongTempFloatingWindow(display_filter)),
            "TempFloatingWindow" => Ok(TempFloatingWindow(display_filter)),
            "Api" => Ok(Api(display_filter)),
            "NvimNotify" => Ok(NvimNotify(display_filter)),
            _ => Err(SniprunError::InternalError(
                "Invalid display type: ".to_string() + s,
            )),
        }
    }
}

impl fmt::Display for DisplayFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            OnlyOk => "Ok",
            OnlyErr => "Err",
            Both => "",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for DisplayType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match &self {
            DisplayType::Classic(f) => "Classic".to_string() + &f.to_string(),
            DisplayType::VirtualText(f) => "VirtualText".to_string() + &f.to_string(),
            DisplayType::Terminal(f) => "Terminal".to_string() + &f.to_string(),
            DisplayType::TerminalWithCode(f) => "TerminalWithCode".to_string() + &f.to_string(),
            DisplayType::LongTempFloatingWindow(f) => {
                "LongTempFloatingWindow".to_string() + &f.to_string()
            }
            DisplayType::TempFloatingWindow(f) => "TempFloatingWindow".to_string() + &f.to_string(),
            DisplayType::Api(f) => "Api".to_string() + &f.to_string(),
            DisplayType::NvimNotify(f) => "NvimNotify".to_string() + &f.to_string(),
        };
        write!(f, "{}", name)
    }
}

pub fn display(result: Result<String, SniprunError>, nvim: Arc<Mutex<Neovim>>, data: &DataHolder) {
    let mut display_type = data.display_type.clone();
    display_type.sort();
    display_type.dedup(); //now only uniques display types

    // remove transparently incompatible/redundant displays
    for filter in [OnlyOk, OnlyErr, Both] {
        if display_type.contains(&TerminalWithCode(filter)) {
            display_type.retain(|dt| dt != &Terminal(filter));
        }
        if display_type.contains(&TempFloatingWindow(filter)) {
            display_type.retain(|dt| dt != &LongTempFloatingWindow(filter));
        }
    }

    info!("Display type chosen: {:?}", display_type);
    for dt in display_type.iter() {
        match dt {
            Classic(f) => {
                return_message_classic(&result, &nvim, &data.return_message_type, data, *f)
            }
            VirtualText(f) => display_virtual_text(&result, &nvim, data, *f),
            Terminal(f) => display_terminal(&result, &nvim, data, *f),
            TerminalWithCode(f) => display_terminal_with_code(&result, &nvim, data, *f),
            LongTempFloatingWindow(f) => display_floating_window(&result, &nvim, data, true, *f),
            TempFloatingWindow(f) => display_floating_window(&result, &nvim, data, false, *f),
            Api(f) => send_api(&result, &nvim, data, *f),
            NvimNotify(f) => display_nvim_notify(&result, &nvim, data, *f),
        }
    }
}

pub fn display_nvim_notify(
    message: &Result<String, SniprunError>,
    nvim: &Arc<Mutex<Neovim>>,
    data: &DataHolder,
    filter: DisplayFilter,
) {
    let res = match (message, filter) {
        (Ok(result), OnlyOk) | (Ok(result), Both) => nvim.lock().unwrap().command(&format!(
            "lua require\"sniprun.display\".display_nvim_notify(\"{}\", true)",
            no_output_wrap(result, data, &DisplayType::NvimNotify(filter)).replace('\n', "\\\n"),
        )),
        (Err(result), OnlyErr) | (Err(result), Both) => nvim.lock().unwrap().command(&format!(
            "lua require\"sniprun.display\".display_nvim_notify(\"{}\", false)",
            no_output_wrap(&result.to_string(), data, &DisplayType::NvimNotify(filter))
                .replace('\n', "\\\n"),
        )),
        _ => Ok(()),
    };
    info!("display notify res = {:?}", res);
}

pub fn send_api(
    message: &Result<String, SniprunError>,
    nvim: &Arc<Mutex<Neovim>>,
    data: &DataHolder,
    filter: DisplayFilter,
) {
    let res = match (message, filter) {
        (Ok(result), OnlyOk) | (Ok(result), Both) => {
            let mut nvim_instance = nvim.lock().unwrap();
            nvim_instance.command(&format!(
                "lua require\"sniprun.display\".send_api(\"{}\", true)",
                no_output_wrap(result, data, &DisplayType::Api(filter)).replace('\n', "\\\n"),
            ))
        }
        (Err(result), OnlyErr) | (Err(result), Both) => {
            let mut nvim_instance = nvim.lock().unwrap();
            nvim_instance.command(&format!(
                "lua require\"sniprun.display\".send_api(\"{}\", false)",
                no_output_wrap(&result.to_string(), data, &DisplayType::Api(filter))
                    .replace('\n', "\\\n"),
            ))
        }
        _ => Ok(()),
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
    filter: DisplayFilter,
) {
    info!("range is : {:?}", data.range);
    let namespace_id = nvim.lock().unwrap().create_namespace("sniprun").unwrap();
    if (filter == OnlyOk) != result.is_ok() {
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
    let res = match (result, filter) {
        (Ok(message_ok), OnlyOk) | (Ok(message_ok), Both) => {
            if shorten_ok(&no_output_wrap(
                message_ok,
                data,
                &DisplayType::VirtualText(filter),
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
                    &DisplayType::VirtualText(filter)
                )),
                hl_ok
            ))
        }
        (Err(message_err), OnlyErr) | (Err(message_err), Both) => {
            if shorten_err(&no_output_wrap(
                &message_err.to_string(),
                data,
                &DisplayType::VirtualText(filter),
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
                    &DisplayType::VirtualText(filter)
                )),
                hl_err
            ))
        }
        _ => Ok(()),
    };
    info!("done displaying virtual text, {:?}", res);
}

pub fn display_terminal(
    message: &Result<String, SniprunError>,
    nvim: &Arc<Mutex<Neovim>>,
    data: &DataHolder,
    filter: DisplayFilter,
) {
    info!("data_bloc = {}", data.current_bloc);
    let a = data.current_bloc.lines();
    info!("length = {}", a.count());
    info!("display terminal, with filter: = {:?}", filter);
    let res = match (message, filter) {
        (Ok(result), OnlyOk) | (Ok(result), Both) => nvim.lock().unwrap().command(&format!(
            "lua require\"sniprun.display\".write_to_term(\"{}\", true)",
            no_output_wrap(result, data, &DisplayType::Terminal(filter)).replace('\n', "\\\n"),
        )),
        (Err(result), OnlyErr) | (Err(result), Both) => nvim.lock().unwrap().command(&format!(
            "lua require\"sniprun.display\".write_to_term(\"{}\", false)",
            no_output_wrap(&result.to_string(), data, &DisplayType::Terminal(filter))
                .replace('\n', "\\\n"),
        )),
        _ => Ok(()),
    };
    info!("display terminal res = {:?}", res);
}

pub fn display_terminal_with_code(
    message: &Result<String, SniprunError>,
    nvim: &Arc<Mutex<Neovim>>,
    data: &DataHolder,
    filter: DisplayFilter,
) {
    let res = match (message, filter) {
        (Ok(result), OnlyOk) | (Ok(result), Both) => nvim.lock().unwrap().command(&format!(
            "lua require\"sniprun.display\".write_to_term(\"{}\\n{}\", true)",
            cleanup_and_escape(
                &format!("\n{}", &data.current_bloc)
                    .unindent()
                    .lines()
                    .fold("".to_string(), |cur_bloc, line_in_bloc| {
                        cur_bloc + "> " + line_in_bloc + "\n"
                    })
            )
            .replace('\n', "\\\n"),
            no_output_wrap(result, data, &DisplayType::TerminalWithCode(filter))
                .replace('\n', "\\\n"),
        )),
        (Err(result), OnlyErr) | (Err(result), Both) => nvim.lock().unwrap().command(&format!(
            "lua require\"sniprun.display\".write_to_term(\"{}\\n{}\", false)",
            cleanup_and_escape(
                &format!("\n{}", &data.current_bloc)
                    .unindent()
                    .lines()
                    .fold("".to_string(), |cur_bloc, line_in_bloc| {
                        cur_bloc + "> " + line_in_bloc + "\n"
                    })
            )
            .replace('\n', "\\\n"),
            no_output_wrap(
                &result.to_string(),
                data,
                &DisplayType::TerminalWithCode(filter)
            )
            .replace('\n', "\\\n"),
        )),
        _ => Ok(()),
    };
    info!("display terminal res = {:?}", res);
}

pub fn display_floating_window(
    message: &Result<String, SniprunError>,
    nvim: &Arc<Mutex<Neovim>>,
    data: &DataHolder,
    long_only: bool,
    filter: DisplayFilter,
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

    let res = match (message, filter) {
        (Ok(result), OnlyOk) | (Ok(result), Both) => nvim.lock().unwrap().command(&format!(
            "lua require\"sniprun.display\".fw_open({},{},\"{}\", true)",
            row - 1,
            col,
            no_output_wrap(
                &result.to_string(),
                data,
                &DisplayType::TempFloatingWindow(filter)
            )
            .replace('\n', "\\\n"),
        )),
        (Err(result), OnlyErr) | (Err(result), Both) => nvim.lock().unwrap().command(&format!(
            "lua require\"sniprun.display\".fw_open({},{},\"{}\", false)",
            row - 1,
            col,
            no_output_wrap(
                &result.to_string(),
                data,
                &DisplayType::TempFloatingWindow(filter)
            )
            .replace('\n', "\\\n"),
        )),
        _ => Ok(()),
    };
    info!("display floating window res = {:?}", res);
}

pub fn return_message_classic(
    message: &Result<String, SniprunError>,
    nvim: &Arc<Mutex<Neovim>>,
    rmt: &ReturnMessageType,
    data: &DataHolder,
    filter: DisplayFilter,
) {
    match (message, filter) {
        (Ok(answer_ok), OnlyOk) | (Ok(answer_ok), Both) => {
            //make sure there is no lone "
            let answer_str = no_output_wrap(answer_ok, data, &DisplayType::Classic(filter));
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
        (Err(e), OnlyErr) | (Err(e), Both) => match rmt {
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
        _ => (),
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
