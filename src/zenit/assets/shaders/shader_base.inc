#ifndef SHADER_BASE_INC
#define SHADER_BASE_INC

// For shared use of vertex->fragment values (see example_triangle for sample usage)
#ifdef VERTEX
    #define vout out
#else
    #define vout in
#endif

#endif
