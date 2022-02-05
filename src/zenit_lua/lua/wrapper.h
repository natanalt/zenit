//! Wrapper for Rust bindgen

#include <lua.h>
#include <lauxlib.h>
#include <lualib.h>

// We also want access to internal Lua functionality, in order to allow a custom
// compiled chunk loader to be implemented from Rust.
#include "src/lfunc.h"
#include "src/lstring.h"
#include "src/lmem.h"
#include "src/ldo.h"

// Our very own lovely Lua extensions
#include "src/lzenit.h"
