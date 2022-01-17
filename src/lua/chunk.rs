//! Custom Lua chunk parser.
//!
//! As all SWBF2 scripts come in a compiled form, a chunk loader is needed. The C Lua implementation
//! provides one, obviously, but it comes with one serious limitation - it can only load chunks
//! compiled for a specific version of the interpreter. To be exact, it's dependent on variables
//! like sizes of `int` or `size_t` datatypes, that are not going to be exact across architectures.
//!
//! Since PC SWBF2 chunks were compiled for a 32-bit x86 build of Lua, this means that portability
//! becomes a massive issue - even between 32-bit and 64-bit x86, the scripts aren't loadable by
//! themselves.
//!
//! The solution that Zenit goes for is implementing a custom compiled chunk parser. Internally,
//! all compiled chunks follow the same structure, regardless of the architecture they were compiled
//! for. Therefore this parser reads the x86-32 targeting chunks, and hands them to Lua in whatever
//! form it'll be comfortable with - regardless of the target architecture of Zenit itself.
//!
//! This code pretty much copies the behavior of `lundump.c` from Lua.
//! Safety not guaranteed.
//!

// This all may and probably will break on builds where int is 64-bit
// i despise C

use super::{ffi, LuaState};
use crate::{error_if, AnyResult};
use byteorder::{ReadBytesExt, LE};
use std::{
    ffi::CStr,
    io::{Cursor, Read, Seek, SeekFrom},
    mem::size_of,
    ptr,
};
use thiserror::Error;

#[rustfmt::skip]
pub const EXPECTED_HEADER: [u8; 18] = [
    0x1b, b'L', b'u', b'a',
    0x50, // Lua version
    1, // Endianness (1 = little)
    4, // int size
    4, // size_t size
    4, // Instruction size
    6, // OP size
    8, // A size
    9, // B size
    9, // C size
    4, // number type size
    0x3b, 0xaf, 0xef, 0x4b, // Test number (31415926)
];

#[derive(Debug, Error, Clone, Copy)]
pub enum ParseError {
    #[error("invalid chunk header")]
    InvalidHeader,
    #[error("sudden chunk end of stream")]
    IncorrectSize,
    #[error("unspecified chunk format error")]
    FormatError,
    #[error("unspecified Lua API side error")]
    LuaApiError,
}

/// Loads the chunk into given Lua state and pushes loaded chunk as a function on the top of the Lua
/// stack.
///
/// Overall, this function is equivalent to Lua's `lua_load`, but accepts architecturally foreign
/// chunks (see module documentation).
///
pub fn load_chunk(l: &mut LuaState, blob: &[u8]) -> AnyResult<()> {
    let mut reader = Cursor::new(blob);

    let mut header = [0; 18];
    reader.read(&mut header)?;
    if header != EXPECTED_HEADER {
        return Err(ParseError::InvalidHeader)?;
    }

    unsafe {
        let proto = read_function(l, &mut reader)?;
        if !ffi::luaZE_pushlclosure(l.raw(), proto) {
            return Err(ParseError::LuaApiError)?;
        }
    }

    Ok(())
}

/// Makes using everything a lot less annoying
trait ErroringReaderExt {
    fn read_u8_c(&mut self) -> AnyResult<u8>;
    fn read_u32_c(&mut self) -> AnyResult<u32>;
    fn read_i32_c(&mut self) -> AnyResult<i32>;
    fn read_f32_c(&mut self) -> AnyResult<f32>;
}

impl ErroringReaderExt for Cursor<&[u8]> {
    fn read_u8_c(&mut self) -> AnyResult<u8> {
        Ok(self.read_u8().map_err(|_| ParseError::IncorrectSize)?)
    }

    fn read_u32_c(&mut self) -> AnyResult<u32> {
        Ok(self
            .read_u32::<LE>()
            .map_err(|_| ParseError::IncorrectSize)?)
    }

    fn read_i32_c(&mut self) -> AnyResult<i32> {
        Ok(self
            .read_i32::<LE>()
            .map_err(|_| ParseError::IncorrectSize)?)
    }

    fn read_f32_c(&mut self) -> AnyResult<f32> {
        Ok(self
            .read_f32::<LE>()
            .map_err(|_| ParseError::IncorrectSize)?)
    }
}

unsafe fn read_function(
    l: &mut LuaState,
    reader: &mut Cursor<&[u8]>,
) -> AnyResult<*mut ffi::Proto> {
    let proto = ffi::luaF_newproto(l.raw());
    error_if!(proto.is_null(), ParseError::LuaApiError);

    // Load header
    (*proto).source = read_lua_string(l, reader)?;
    (*proto).lineDefined = reader.read_i32_c()? as _;
    (*proto).nups = reader.read_u8_c()? as _;
    (*proto).numparams = reader.read_u8_c()? as _;
    (*proto).is_vararg = reader.read_u8_c()? as _;
    (*proto).maxstacksize = reader.read_u8_c()? as _;

    // Load lines
    let line_count = reader.read_i32_c()? as _;
    (*proto).sizelineinfo = line_count;
    (*proto).lineinfo = create_lua_vector::<i32>(l, line_count)?;
    for i in 0..line_count {
        *((*proto).lineinfo.offset(i as isize)) = reader.read_i32_c()? as _;
    }

    // Load locals
    let local_count = reader.read_i32_c()?;
    (*proto).sizelocvars = local_count;
    (*proto).locvars = create_lua_vector::<ffi::LocVar>(l, local_count)?;
    for i in 0..local_count {
        let name = read_lua_string(l, reader)?;
        let start = reader.read_i32_c()?;
        let end = reader.read_i32_c()?;
        *((*proto).locvars.offset(i as _)) = ffi::LocVar {
            varname: name,
            startpc: start,
            endpc: end,
        };
    }

    // Load upvalues
    let upvalue_count = reader.read_i32_c()?;
    if upvalue_count != (*proto).nups as i32 {
        return Err(ParseError::LuaApiError)?;
    }
    (*proto).upvalues = create_lua_vector::<*mut ffi::TString>(l, upvalue_count)?;
    (*proto).sizeupvalues = upvalue_count;
    for i in 0..upvalue_count {
        *((*proto).upvalues.offset(i as _)) = read_lua_string(l, reader)?;
    }

    // Load constants
    let constant_count = reader.read_i32_c()?;
    (*proto).k = create_lua_vector::<ffi::TObject>(l, constant_count)?;
    (*proto).sizek = constant_count;
    for i in 0..constant_count {
        let kind = reader.read_u8_c()?;
        let object = (*proto).k.offset(i as _);
        (*object).tt = kind as _;
        match kind as u32 {
            ffi::LUA_TNUMBER => {
                (*object).value.n = reader.read_f32_c()?;
            }
            ffi::LUA_TSTRING => {
                (*object).value.gc = read_lua_string(l, reader)? as *mut ffi::GCObject;
            }
            ffi::LUA_TNIL => {
                // Nothing needs to be done
            }
            _ => {
                return Err(ParseError::FormatError)?;
            }
        }
    }
    let subfunc_count = reader.read_i32_c()?;
    (*proto).p = create_lua_vector::<*mut ffi::Proto>(l, subfunc_count)?;
    (*proto).sizep = subfunc_count;
    for i in 0..subfunc_count {
        let p = (*proto).p;
        *p.offset(i as _) = read_function(l, reader)?;
    }

    // Load code
    let code_instructions = reader.read_i32_c()?;
    (*proto).code = create_lua_vector::<ffi::Instruction>(l, code_instructions)?;
    (*proto).sizecode = code_instructions;
    for i in 0..code_instructions {
        // Assuming that Instruction is always a u32
        *((*proto).code.offset(i as _)) = reader.read_u32_c()?;
    }

    Ok(proto)
}

/// May return null, if the string is empty. This will not be marked as failure.
unsafe fn read_lua_string(
    l: &mut LuaState,
    reader: &mut Cursor<&[u8]>,
) -> AnyResult<*mut ffi::TString> {
    let len = reader.read_i32_c()?;
    if len == 0 {
        return Ok(ptr::null_mut());
    }

    let position = reader.position() as usize;
    let string_data = &reader.get_ref()[position..position + len as usize];
    let data = CStr::from_bytes_with_nul(string_data)?;
    reader.seek(SeekFrom::Current(len as i64))?;

    // Subtracting 1 to remove the trailing nul byte
    let result = ffi::luaS_newlstr(l.raw(), data.as_ptr(), (len - 1) as _);
    return if result.is_null() {
        Err(ParseError::LuaApiError)?
    } else {
        Ok(result)
    };
}

/// **Will return null** if len is 0
unsafe fn create_lua_vector<T>(l: &mut LuaState, len: i32) -> AnyResult<*mut T> {
    if len == 0 {
        return Ok(ptr::null_mut());
    }

    let vector = ffi::luaM_realloc(
        l.raw(),
        ptr::null_mut(),
        0,
        (len * size_of::<T>() as i32) as ffi::lu_mem,
    ) as *mut T;

    return if !vector.is_null() {
        Ok(vector)
    } else {
        Err(ParseError::LuaApiError)?
    };
}
