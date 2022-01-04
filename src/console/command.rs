use crate::{engine::Engine, AnyResult};
use super::{ConsoleValue, Console, ConsoleValueKind};

pub type ConCommandCallback = dyn Fn(&mut Engine, &[ConsoleValue]) -> AnyResult<()>;

pub struct ConCommand {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ConCommandParameter>,
    pub callback: Box<ConCommandCallback>,
}

impl std::fmt::Debug for ConCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConCommand").field("name", &self.name).finish()
    }
}

#[derive(Debug, Clone)]
pub struct ConCommandParameter {
    pub name: String,
    pub description: String,
    pub kind: ConsoleValueKind,
}

pub struct ConCommandBuilder<'c> {
    parent: &'c mut Console,
    name: String,
    description: String,
    parameters: Vec<ConCommandParameter>,
    callback: Option<Box<ConCommandCallback>>,
}

impl<'c> ConCommandBuilder<'c> {
    pub fn new(parent: &'c mut Console, name: &str, desc: &str) -> Self {
        Self {
            parent,
            name: name.to_string(),
            description: desc.to_string(),
            parameters: Vec::new(),
            callback: None,
        }
    }

    pub fn callback(mut self, callback: Box<ConCommandCallback>) -> Self {
        self.callback = Some(callback);
        self
    }

    pub fn parameter(mut self, name: &str, desc: &str, kind: ConsoleValueKind) -> Self {
        self.parameters.push(ConCommandParameter {
            name: name.to_string(),
            description: desc.to_string(),
            kind,
        });
        self
    }

    pub fn build(self) -> &'c mut Console {
        self.parent.commands.insert(self.name.clone(), ConCommand {
            name: self.name,
            description: self.description,
            parameters: self.parameters,
            callback: self.callback.expect("Command build without callback"),
        });
        self.parent
    }
}

