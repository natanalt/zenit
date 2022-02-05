#ifndef ZENIT_LUA_H
#define ZENIT_LUA_H

// Zenit Lua extensions

#include "lfunc.h"
#include <stdbool.h>

bool luaZE_pushlclosure(lua_State* L, Proto* proto);

#endif
