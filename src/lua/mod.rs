//! The Lua bindings.
//!
//! I'm not using any existing crates, as none of them support a 17 year old version of Lua,
//! understandably so.
//!
//! Most documentation in this module will assume that you, the reader, already know at least a bit
//! about Lua programming, and how C, or in this case Rust code interfaces with the Lua environment,
//! which includes things like the internal Lua stack.
//!

use thiserror::Error;
use crate::AnyResult;
use std::ffi::CStr;

pub mod chunk;
pub use chunk::load_chunk;

pub mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    include!(concat!(env!("OUT_DIR"), "/lua_bindings.rs"));
}

/// Lua state represents a Lua environment, where all functions, variables and other things live.
pub struct LuaState {
    raw: *mut ffi::lua_State,
}

#[derive(Error, Debug, Clone)]
pub enum LuaCallError {
    #[error("Unknown Lua error: {0}")]
    UnknownError(String),
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    #[error("Memory allocation error: {0}")]
    MemoryAllocationError(String),
    #[error("Error while running the error handler: {0}")]
    ErrorHandlerError(String),
}

#[derive(Debug, Clone, Copy)]
pub enum LuaType {
    Nil,
    Number,
    Boolean,
    String,
    Table,
    Function,
    UserData,
    Thread,
    LightUserData,
}

impl LuaType {
    pub fn from_c_constant(value: i32) -> Option<LuaType> {
        match value as u32 {
            ffi::LUA_TNIL => Some(LuaType::Nil),
            ffi::LUA_TNUMBER => Some(LuaType::Number),
            ffi::LUA_TBOOLEAN => Some(LuaType::Boolean),
            ffi::LUA_TSTRING => Some(LuaType::String),
            ffi::LUA_TTABLE => Some(LuaType::Table),
            ffi::LUA_TFUNCTION => Some(LuaType::Function),
            ffi::LUA_TUSERDATA => Some(LuaType::UserData),
            ffi::LUA_TTHREAD => Some(LuaType::Thread),
            ffi::LUA_TLIGHTUSERDATA => Some(LuaType::LightUserData),
            _ => None,
        }
    }
}

impl LuaState {
    /// Creates a new Lua state.
    ///
    /// ## Panics
    /// Panics if the creation fails. This can only mean one thing - an in... -ternal
    /// allocation failure.
    ///
    pub fn new() -> Self {
        unsafe {
            let raw = ffi::lua_open();
            if !raw.is_null() {
                LuaState { raw }
            } else {
                panic!("Lua state opening failure. Out of memory?")
            }
        }
    }

    /// Returns a pointer to the underlying C `lua_State` instance.
    pub fn raw(&self) -> *mut ffi::lua_State {
        self.raw
    }

    pub fn call(&mut self, param_count: i32, result_count: i32) -> AnyResult<()> {
        //unsafe {
        //    match ffi::lua_pcall(self.raw(), param_count, result_count, 0) as u32 {
        //        0 => Ok(()),
        //        ffi::LUA_ERRRUN => Err(LuaCallError::RuntimeError(
        //            self.pop_string().unwrap_or("<unknown error>".to_string()),
        //        ))?,
        //        ffi::LUA_ERRMEM => Err(LuaCallError::MemoryAllocationError(
        //            self.pop_string().unwrap_or("<unknown error>".to_string()),
        //        ))?,
        //        ffi::LUA_ERRERR => Err(LuaCallError::ErrorHandlerError(
        //            self.pop_string().unwrap_or("<unknown error>".to_string()),
        //        ))?, // Note, we do not implement error handlers rn
        //        _ => Err(LuaCallError::UnknownError(
        //            self.pop_string().unwrap_or("<unknown error>".to_string()),
        //        ))?,
        //    }
        //}
        todo!()
    }
}

/// Stack manipulation
impl LuaState {
    pub fn pop(&mut self, amount: u32) {
        self.set_stack_top(-(amount as i32) - 1);
    }

    pub fn set_stack_top(&mut self, idx: i32) {
        unsafe {
            ffi::lua_settop(self.raw(), idx);
        }
    }

    pub fn push_copy_of(&mut self, stack_idx: i32) {
        unsafe {
            ffi::lua_pushvalue(self.raw(), stack_idx);
        }
    }

    pub fn remove_from_stack(&mut self, idx: i32) {
        unsafe {
            ffi::lua_remove(self.raw(), idx);
        }
    }

    pub fn move_from_stack_top(&mut self, idx: i32) {
        unsafe {
            ffi::lua_insert(self.raw(), idx);
        }
    }

    pub fn replace_from_stack_top(&mut self, idx: i32) {
        unsafe {
            ffi::lua_replace(self.raw(), idx);
        }
    }

    pub fn get_type(&self, idx: i32) -> LuaType {
        unsafe {
            LuaType::from_c_constant(ffi::lua_type(self.raw(), idx))
                .expect("Invalid lua_type return")
        }
    }

    pub fn push_nil(&mut self) {
        unsafe {
            ffi::lua_pushnil(self.raw());
        }
    }

    pub fn push_number(&mut self, num: f32) {
        unsafe {
            ffi::lua_pushnumber(self.raw(), num);
        }
    }

    pub fn push_string(&mut self, str: &str) {
        unsafe {
            ffi::lua_pushlstring(self.raw(), str.as_ptr() as _, str.len() as _);
        }
    }

    pub fn push_light_user_data(&mut self, ptr: usize) {
        unsafe {
            ffi::lua_pushlightuserdata(self.raw(), ptr as _);
        }
    }

    pub fn peek_number(&mut self, idx: i32) -> f32 {
        unsafe { ffi::lua_tonumber(self.raw(), idx) }
    }

    pub fn pop_number(&mut self) -> f32 {
        let result = self.peek_number(-1);
        self.pop(1);
        result
    }

    /// Returns None if there are no stringables on the stack
    pub fn peek_string(&mut self, idx: i32) -> AnyResult<Option<String>> {
        unsafe {
            let raw = ffi::lua_tostring(self.raw(), idx);
            Ok(if !raw.is_null() {
                Some(CStr::from_ptr(raw as _).to_str()?.to_string())
            } else {
                None
            })
        }
    }

    pub fn pop_string(&mut self) -> AnyResult<Option<String>> {
        Ok(self.peek_string(-1)?.and_then(|x| {
            self.pop(1);
            Some(x)
        }))
    }

    pub fn peek_string_or(&mut self, idx: i32, default: &str) -> String {
        self.peek_string(idx)
            .ok()
            .flatten()
            .unwrap_or_else(|| default.to_string())
    }

    pub fn push_globals_table(&mut self) {
        unsafe {
            ffi::lua_gettable(self.raw(), ffi::LUA_GLOBALSINDEX);
        }
    }
}

/// Library openings
impl LuaState {
    /// Opens the basic part of the Lua standard library
    pub fn open_base_library(&mut self) {
        unsafe {
            ffi::luaopen_base(self.raw());
        }
    }

    /// Opens the string part of the Lua standard library
    pub fn open_string(&mut self) {
        unsafe {
            ffi::luaopen_string(self.raw());
        }
    }

    /// Opens the table part of the Lua standard library
    pub fn open_table(&mut self) {
        unsafe {
            ffi::luaopen_table(self.raw());
        }
    }

    /// Opens the math part of the Lua standard library
    pub fn open_math_library(&mut self) {
        unsafe {
            ffi::luaopen_math(self.raw());
        }
    }

    /// Opens the IO and OS part of the Lua standard library
    pub fn open_io_library(&mut self) {
        unsafe {
            ffi::luaopen_io(self.raw());
        }
    }

    /// Opens the debug part of the Lua standard library
    pub fn open_debug_library(&mut self) {
        unsafe {
            ffi::luaopen_debug(self.raw());
        }
    }
}

impl Drop for LuaState {
    fn drop(&mut self) {
        unsafe {
            ffi::lua_close(self.raw);
        }
    }
}
