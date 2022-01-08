//! Shell is the main menu of SWBF2, mostly scripted by Lua.
//!

use crate::engine::{Engine, GameState};

#[derive(Debug, Clone)]
pub struct ShellState {

}

impl ShellState {
    pub fn new(engine: &mut Engine) -> Self {
        Self {}
    }

    pub fn frame(mut self, engine: &mut Engine) -> GameState {
        GameState::Shell(self)
    }
}
