
@group(0) @binding(0)
var<uniform> u_screen_size: vec2<f32>;

@group(1) @binding(0)
var u_texture: texture_2d<f32>;
@group(1) @binding(1)
var u_sampler: sampler;

struct VertexInput {
    @location(0) a_position: vec2<f32>,
    @location(1) a_uv: vec2<f32>,
    @location(2) a_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) v_position: vec4<f32>,
    @location(0) v_uv: vec2<f32>,
    @location(1) v_color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    // We're not supporting multi-viewport anytime soon, so we'll just hackily transform the vertex position
    // to fit the render target.
    var final_position = in.a_position;
    final_position *= vec2<f32>(1.0, -1.0);             // Vertically flip the vertices
    final_position *= vec2<f32>(2.0) / u_screen_size;   // Convert the coordinate space into ([0;2], [0;-2])
    final_position += vec2<f32>(-1.0, 1.0);             // And shift them so (0,0) is actually middle

    var out: VertexOutput;
    out.v_position = vec4<f32>(final_position, 0.0, 1.0);
    out.v_uv = in.a_uv;
    out.v_color = in.a_color;
    return out;
}

fn srgb_to_linear(color: vec4<f32>) -> vec4<f32> {
    let selector = ceil(color.rgb - 0.04045); // 0 if under value, 1 if over
    let under = color.rgb / 12.92;
    let over = pow((color.rgb + 0.055) / 1.055, vec3<f32>(2.4));
    let result = mix(under, over, selector);
    return vec4<f32>(result, color.a);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return srgb_to_linear(in.v_color) * textureSample(u_texture, u_sampler, in.v_uv);
}
