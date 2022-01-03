
> Zenit Notes
  -----------
  Note: the notes are outdated, as the Lua approach has been changing a
  lot. Someday I'll rewrite them lol ~nat

  This is slightly modified Lua 5.0.2 (same version as used by SWBF2, but
  Pandemic applied slight mods), just with most non-interpreter files
  removed. The only files remaining are:
   * core Lua sources
   * public Lua headers
   * the Lua standard library
  
  An important factor is the target architecture. SWBF2 ships its
  Lua scripts as precompiled chunks, which contain their VM assumptions in
  their headers. Those are, unfortunately, target specific, and depend on
  values of int and size_t (4 bytes each).

  A second, smaller modification was reformatting the source tree using
  clang-foramt, with the default Webkit preset, to make it a bit easier
  on the eyes.

  Build used in SWBF2 may have been at least partially modified:
   * size of the Lua number type was changed from 8 to 4 (double -> float
     most likely)
  
  Lua is compiled from the build.rs script in the repo root directory.

  Notes:
   * apply macro TRUST_BINARIES

  If you want to read the documentation of Lua 5.0.2, you can download the
  official distribution and get it from there:
    https://www.lua.org/ftp/lua-5.0.2.tar.gz


This is Lua 5.0.
See HISTORY for a summary of changes since the last released version.

* What is Lua?
  ------------
  Lua is a powerful, light-weight programming language designed for extending
  applications. Lua is also frequently used as a general-purpose, stand-alone
  language. Lua is free software.

  For complete information, visit Lua's web site at http://www.lua.org/ .
  For an executive summary, see http://www.lua.org/about.html .

  Lua has been used in many different projects around the world.
  For a short list, see http://www.lua.org/uses.html .

* Availability
  ------------
  Lua is freely available for both academic and commercial purposes.
  See COPYRIGHT and http://www.lua.org/license.html for details.
  Lua can be downloaded from its official site http://www.lua.org/ and
  several other sites aroung the world. For a complete list of mirror sites,
  see http://www.lua.org/mirrors.html .

* Installation
  ------------
  Lua is implemented in pure ANSI C, and compiles unmodified in all known
  platforms that have an ANSI C compiler. Under Unix, simply typing "make"
  should work. See INSTALL for detailed instructions.

* Contacting the authors
  ----------------------
  Send your comments, questions, and bug reports to lua@tecgraf.puc-rio.br.
  For more information about the authors, see http://www.lua.org/authors.html .
  For reporting bugs, try also the mailing list: lua-l@tecgraf.puc-rio.br.
  For more information about this list, including instructions on how to
  subscribe and access the archives, see http://www.lua.org/lua-l.html .

* Origin
  ------
  Lua is developed at Tecgraf, the Computer Graphics Technology Group
  of PUC-Rio (the Pontifical Catholic University of Rio de Janeiro in Brazil).
  Tecgraf is a laboratory of the Department of Computer Science.

(end of README)
