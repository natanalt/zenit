# Lua tests
There are various tests of the VM in this directory. All of them involve loading an external Lua chunk, executing it, and testing for some kind of result. What that result test involves depends on the test itself.

All of the .lua files in this directory were compiled externally using the reference Lua 5.0.2 implementation:
 * Lua was compiled with Visual Studio, for x86**-32**
 * Compiled with 32-bit floats as the `number` type
 * Basically it's the same format as Battlefront 2 uses for precompiled scripts

Even though the reference implementation doesn't load chunks compiled by foreign interpreters, Zenit must be able to accept them on any platform.

If you modify any of the .lua files in this directory, you should recompile them.
