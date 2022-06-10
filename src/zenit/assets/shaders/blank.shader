description = "Blank shader"

vertex {
    void main() {
        gl_Position = vec4(0.0);
    }
}

fragment {
    layout (location = 0) out vec4 o_color;
    void main() {}
}
