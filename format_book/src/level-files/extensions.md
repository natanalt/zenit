# Zenit-specific extensions
Zenit ships a whole set of built-in dependencies via a file called `zenit_builtin.lvl`. In ordinary
engine builds, this file is compiled into the engine, but it does still follow the format of a
regular level file, albeit with a few extensions.

This document describes what "non-standard" behavior is implemented.

## `tex_` (modifications)
 * Added an `R8G8B8` format (equivalent to `D3DFMT_R8G8B8 = 20 = 0x14`)
 * Texture filtering can be disabled by including at least one `NFLT` node within a *texture format* node (`FMT_`). The `NFLT` node contents are ignored by the engine.

## `WGSL` (new node)
The `WGSL` node stores WGSL shader code.

It follows the structure:
 * `WGSL`
   * `NAME` - null-terminated UTF-8 shader name
   * `CODE` - null-terminated UTF-8 shader code