struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) texture_coord: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) texture_coord: vec2<f32>
};

struct Uniforms {
    matr: mat4x4<f32>,
    z: f32,
    grid: u32,
};
@group(1) @binding(0)
var<uniform> unifs: Uniforms;

@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = unifs.matr * vec4(input.position, 0.0, 1.0); // TODO pvm and z uniforms
    out.texture_coord = input.texture_coord;
    return out;
}

// Frag
@group(0) @binding(0)
var text: texture_2d<f32>;
@group(0)@binding(1)
var sampl: sampler;

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    var rgb_color = textureSample(text, sampl, in.texture_coord); // TODO: texture sampler
//    var temp = ((rgb_color / 255.0 + 0.055) / 1.055);
//    var srgb_color: vec4<f32>;
//    srgb_color.r = pow(temp.r, 2.4);
    rgb_color.a = 1.0;
    return rgb_color;
}//    srgb_color.g = pow(temp.g, 2.4);
//    srgb_color.b = pow(temp.b, 2.4);
//    srgb_color.a = rgb_color.a;
    //temp
