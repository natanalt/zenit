#include "lzenit.h"
#include "lgc.h"
#include "ldo.h"

static void pushlclosure_impl(lua_State* L, Proto* proto)
{
    luaC_checkGC(L);
    Closure* cl = luaF_newLclosure(L, 0, gt(L));
    cl->l.p = proto;
    setclvalue(L->top, cl);
    incr_top(L);
}

bool luaZE_pushlclosure(lua_State* L, Proto* proto)
{
    lua_lock(L);
    int status = luaD_rawrunprotected(L, pushlclosure_impl, proto);
    lua_unlock(L);
    return status == 0;
}
