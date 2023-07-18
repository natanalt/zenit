
@group(0) @binding(0)
var<uniform> camera: CameraBuffer;
@group(1) @binding(0)
var cubemap_texture: texture_cube<f32>;
@group(1) @binding(1)
var cubemap_sampler: sampler;

struct SkyboxOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec3<f32>,
}

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> SkyboxOutput {
    var output: SkyboxOutput;
    output.position =
        camera.projection
        * camera.world_to_view
        * vec4(position, 1.0);
    output.uv = position;
    return output;
}

@fragment
fn fs_main(input: SkyboxOutput) -> @location(0) vec4<f32> {
    return textureSample(cubemap_texture, cubemap_sampler, input.uv);
}
