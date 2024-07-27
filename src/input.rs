use crate::error::SniprunError;
use neovim_lib::{Neovim, NeovimApi};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub fn vim_input_ask(message: &str, nvim: &Arc<Mutex<Neovim>>) -> Result<String, SniprunError> {
    nvim.lock()
        .unwrap()
        .session
        .set_timeout(Duration::from_secs(20));
    let res = nvim.lock().unwrap().command_output(&format!(
        "lua require\"sniprun.input\".vim_input(\"{}\")",
        message.replace('\n', "\\n"),
    ));
    match res {
        Err(_) => Err(SniprunError::CustomError(
            "Timeout waiting for user input".into(),
        )),
        Ok(input) => Ok(input),
    }
}
