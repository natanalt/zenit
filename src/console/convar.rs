use std::mem::discriminant;
use log::{info, error};
use crate::{engine::Engine, AnyResult};
use super::{ConsoleValue, Console};

// TODO: move convars into a procmacro-based system

#[derive(Debug)]
pub struct ConVar {
    /// Name of the variable
    pub name: String,
    /// Description of what it does
    pub description: String,
    /// If true, the user will be told that changing this value will only apply after restart
    pub requires_restart: bool,
    /// Default value
    pub default_value: ConsoleValue,
    /// Current value of the convar
    pub value: ConsoleValue,
    /// Every listener of the convar
    pub listeners: Vec<ConVarListener>,
}

impl ConVar {
    pub fn update(&mut self, engine: &mut Engine, new_value: ConsoleValue) {
        let expected_kind = discriminant(&self.default_value);
        if discriminant(&new_value) != expected_kind {
            error!("Invalid value type, expected {:?}", expected_kind);
            return;
        }

        info!("Update `{}` to: {}", &self.name, &new_value);

        self.value = new_value;
        for listener in &self.listeners {
            (listener.callback)(engine, self).unwrap_or_else(|err| {
                error!("An error occurred while calling listener `{}`", &listener.source);
                error!("Details: {}", err);
            });
        }
    }
}

pub struct ConVarListener {
    /// Source of the convar, like "Renderer" or "Data loader"
    pub source: String,
    /// The actual callback
    pub callback: Box<dyn Fn(&mut Engine, &ConVar) -> AnyResult<()>>,
}

impl std::fmt::Debug for ConVarListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConVarListener").field("source", &self.source).finish()
    }
}

pub struct ConVarBuilder<'c> {
    parent: &'c mut Console,
    name: String,
    description: String,
    requires_restart: bool,
    default_value: Option<ConsoleValue>,
    listeners: Vec<ConVarListener>,
}

impl<'c> ConVarBuilder<'c> {
    pub fn new(parent: &'c mut Console, name: &str, desc: &str) -> Self {
        Self {
            parent,
            name: name.to_string(),
            description: desc.to_string(),
            requires_restart: false,
            default_value: None,
            listeners: Vec::new(),
        }
    }

    pub fn requires_restart(mut self) -> Self {
        self.requires_restart = true;
        self
    }

    pub fn default_value(mut self, value: ConsoleValue) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn add_listener(mut self, listener: ConVarListener) -> Self {
        self.listeners.push(listener);
        self
    }

    pub fn build(self) -> &'c mut Console {
        let default = self.default_value.expect("ConVarBuilder must set a default value");
        self.parent.convars.insert(self.name.clone(), ConVar {
            name: self.name,
            description: self.description,
            requires_restart: self.requires_restart,
            default_value: default.clone(),
            value: default,
            listeners: self.listeners,
        });
        self.parent
    }
}
