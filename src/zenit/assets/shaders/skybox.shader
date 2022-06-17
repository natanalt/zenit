description = "Skybox shader"

[bindings]
camera = 0
skybox_texture = 1

shared {
    #define CAMERA_BUFFER_BINDING 0
    #define SKYBOX_TEXTURE_BINDING 1

    #include "shader_base.inc"

    layout (location = 0) vout vec3 f_coords;
}

vertex {
    layout (location = 0) in vec3 a_pos;

    #include "shared_camera.inc"

    void main() {
        f_coords = a_pos;
        gl_Position = camera.projection * camera.world_to_view * vec4(a_pos, 1.0);
    }
}

fragment {
    layout (location = 0) out vec4 o_color;

    layout (binding = SKYBOX_TEXTURE_BINDING) uniform samplerCube u_skybox; 

    void main() {
        o_color = texture(u_skybox, f_coords);
    }
}
