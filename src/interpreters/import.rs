use crate::error::SniprunError;
use crate::interpreter::{Interpreter, SupportLevel};
use crate::DataHolder;
use log::info;

use lazy_static::lazy_static;
use regex::Regex;
use std::fs::{write, DirBuilder, File};
use std::io::prelude::*;
use std::process::Command;

//python-specific
use pyo3::types::PyDict;

//indentation
use unindent::unindent;
