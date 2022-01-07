use crate::{lua::{LuaState, self}, AnyResult};

/// Represents a compiled Lua chunk
pub struct ScriptResource {
    /// Name of the script
    pub name: String,
    /// Data byte present in the game files (usually it's 1, no idea what it means)
    pub info: u8,
    /// Chunk data
    pub chunk: Vec<u8>,
}

impl ScriptResource {
    /// Attempts to load this chunk into specified Lua state. On success, it's pushed to the top
    /// of the Lua stack as a function, just like C `lua_load` does.
    pub fn load(&self, l: &mut LuaState) -> AnyResult<()> {
        lua::load_chunk(l, &self.chunk)?;
        Ok(())
    }
}
