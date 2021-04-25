use crate::error::SniprunError;
use crate::interpreter::{Interpreter, InterpreterUtils, ReplLikeInterpreter, SupportLevel};
use crate::DataHolder;
use log::info;
use serde_json;

use crate::interpreters;
use crate::iter_types;

use std::fs::{write, DirBuilder, File};
use std::io::prelude::*;
use std::process::Command;

use neovim_lib::NeovimApi;


//indentation
use unindent::unindent;

// use jupyter_client::Client;
// use std::collections::HashMap;
