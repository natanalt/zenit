// TOML metadata
// (line starting with // are automatically stripped)
// (this doesn't apply after the line's beginning, where you have to use standard TOML # for comments)

// The metadata is largely arbitrary and doesn't have a well-defined structure.
// Some shaders may store information about, for example, binding indices, so that
// the main renderer code doesn't have to hardcode them internally.

// (despite the description, this shader can actually be used for more than just triangles ;) )
description = "Example triangle shader"

// Shared data is pasted at the beginning of all other shaders
shared {
    #include "shader_base.inc"

    layout (location = 0) vout vec4 f_color;
}

vertex {
    layout (location = 0) in vec2 a_position;
    layout (location = 1) in vec4 a_color;

    void main() {
        gl_Position = vec4(a_position, 0.0, 1.0);
        f_color = a_color;
    }
}

fragment {
    layout (location = 0) out vec4 o_color;

    void main() {
        o_color = f_color;
    }
}
