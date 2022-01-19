use bevy::reflect::TypeUuid;
use crate::{lua::{LuaState, self}, AnyResult};

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "c11cf49c-73fa-4d45-8464-7433d1a34246"]
pub struct CompiledScript {
    pub name: String,
    pub mysterious_info: u8,
    pub data: Vec<u8>,
}

impl CompiledScript {
    pub fn push_chunk(&self, l: &mut LuaState) -> AnyResult<()> {
        Ok(lua::load_chunk(l, &self.data)?)
    }
}
