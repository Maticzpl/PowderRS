struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) color: vec4<f32>
};

struct Uniforms {
    transform: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> unifs: Uniforms;

@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = unifs.transform * vec4(input.position, 0.0, 1.0); //
    out.color = input.color;
    return out;
}

// Frag

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    return in.color;
}
