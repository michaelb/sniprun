use crate::error::SniprunError;
use crate::interpreter::{Interpreter, InterpreterUtils, ReplLikeInterpreter, SupportLevel};
use crate::DataHolder;
use log::info;
use serde_json;

use crate::interpreters;
use crate::iter_types;

use std::fs::{write, DirBuilder, File};
use std::process::Command;


use neovim_lib::NeovimApi;

use std::env;

use crate::daemonizer::{daemon,Fork};

//indentation
use unindent::unindent;

use std::io::prelude::*;

// use jupyter_client::Client;
// use std::collections::HashMap;
