//! Console stuff, Source engine style
//!

use self::{
    command::{ConCommand, ConCommandBuilder},
    convar::ConVarBuilder,
};
use std::{collections::HashMap, fmt::Display};

pub mod command;
pub mod convar;
pub mod parser;
pub use convar::{ConVar, ConVarListener};

#[derive(Debug, Clone, Copy)]
pub enum ConsoleValueKind {
    Bool,
    Int32,
    Float32,
    String,
}

#[derive(Debug, Clone)]
pub enum ConsoleValue {
    Bool(bool),
    Int32(i32),
    Float32(f32),
    String(String),
}

impl ConsoleValue {
    pub fn kind(&self) -> ConsoleValueKind {
        match self {
            ConsoleValue::Bool(_) => ConsoleValueKind::Bool,
            ConsoleValue::Int32(_) => ConsoleValueKind::Int32,
            ConsoleValue::Float32(_) => ConsoleValueKind::Float32,
            ConsoleValue::String(_) => ConsoleValueKind::String,
        }
    }
}

impl Display for ConsoleValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsoleValue::Bool(x) => write!(f, "{}", *x),
            ConsoleValue::Int32(x) => write!(f, "{}", *x),
            ConsoleValue::Float32(x) => write!(f, "{}", *x),
            ConsoleValue::String(x) => write!(f, "\"{}\"", x),
        }
    }
}

#[derive(Debug)]
pub struct Console {
    pub convars: HashMap<String, ConVar>,
    pub commands: HashMap<String, ConCommand>,
}

impl Console {
    pub fn new() -> Self {
        Self {
            convars: HashMap::new(),
            commands: HashMap::new(),
        }
    }

    pub fn read_convar(&self, name: &str) -> Option<&ConsoleValue> {
        self.convars.get(name).and_then(|x| Some(&x.value))
    }

    pub fn read_convar_bool(&self, name: &str) -> Option<bool> {
        self.convars.get(name).and_then(|x| match x.value {
            ConsoleValue::Bool(x) => Some(x),
            _ => None,
        })
    }

    pub fn read_convar_int32(&self, name: &str) -> Option<i32> {
        self.convars.get(name).and_then(|x| match x.value {
            ConsoleValue::Int32(x) => Some(x),
            _ => None,
        })
    }

    pub fn read_convar_float32(&self, name: &str) -> Option<f32> {
        self.convars.get(name).and_then(|x| match x.value {
            ConsoleValue::Float32(x) => Some(x),
            _ => None,
        })
    }

    pub fn read_convar_string(&self, name: &str) -> Option<&str> {
        self.convars.get(name).and_then(|x| match &x.value {
            ConsoleValue::String(x) => Some(x.as_str()),
            _ => None,
        })
    }

    pub fn begin_convar(&mut self, name: &str, desc: &str) -> ConVarBuilder {
        ConVarBuilder::new(self, name, desc)
    }

    pub fn begin_command(&mut self, name: &str, desc: &str) -> ConCommandBuilder {
        ConCommandBuilder::new(self, name, desc)
    }
}
