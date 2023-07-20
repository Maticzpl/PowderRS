struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) texture_coord: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) texture_coord: vec2<f32>,
};

struct Uniforms {
    matr: mat4x4<f32>,
    gui_matr: mat4x4<f32>,
    z: f32,
    grid: u32,
    padding: vec2<f32> // Dummy variable for padding
};

@group(0) @binding(0)
var<uniform> unifs: Uniforms;

@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = unifs.matr * vec4(input.position, unifs.z, 1.0);
    if (input.texture_coord.x > 1.0) // GUI rect
    {
        out.position = unifs.gui_matr * vec4(input.position, unifs.z, 1.0);
    }
    out.texture_coord = input.texture_coord;
    return out;
}

// Frag
@group(1) @binding(0)
var text: texture_2d<f32>;
@group(1) @binding(1)
var sampl: sampler;

@group(2) @binding(0)
var gui_text: texture_2d<f32>;
@group(2) @binding(1)
var gui_sampl: sampler;

fn lerp(a: vec4<f32>, b: vec4<f32>, t: f32) -> vec4<f32>
{
    return ((b - a) * t) + a;
}

fn rand(co: vec2<f32>) -> f32 
{
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453);
}

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    var rgb_color = textureSample(text, sampl, in.texture_coord);

    var ptype: i32 = i32(rgb_color.a * 255.0);
    rgb_color.a = f32(ptype != 0);

    var pixel_size: vec2<f32> = vec2<f32>(1.0) / vec2<f32>(textureDimensions(text, 0));

    var sameNeigh: f32 = 0.0;
    var r: i32 = 1;
    for (var y: i32 = -r; y <= r; y = y + 1) {
        for (var x: i32 = -r; x <= r; x = x + 1) {
            let off: vec2<f32> = vec2(f32(x), f32(y));
            let pixel: vec4<f32> = textureSample(text, sampl, in.texture_coord + off * pixel_size);
            sameNeigh += f32(i32(pixel.a * 255.0) == ptype);
        }
    }
    var neighTotal: f32 = f32(4 * r * r);

    var pos: vec2<f32> = floor(in.texture_coord / pixel_size);

    var rng: f32 = rand(vec2(floor(in.texture_coord / pixel_size) * pixel_size));
    var dustMin: f32 = 0.85;

    var gridBright: f32 = (1.0 / 255.0) * f32((i32(pos.x) + i32(pos.y)) % 2 == 0);
    if (unifs.grid != u32(0)) {
        var isLine: bool =
            (f32(u32(pos.x) % (unifs.grid - u32(1))) == 0.0) ||
            (f32(u32(pos.y) % (unifs.grid - u32(1))) == 0.0);

        gridBright += 2.0 / 255.0 * f32(isLine);
    }

    var gridCol: vec4<f32> = vec4(gridBright, gridBright, gridBright, 0.0);

    // i really dont think this is how i will do this later
    var watrCol: vec4<f32> = lerp(rgb_color * 0.1, rgb_color, sameNeigh / neighTotal) * f32(ptype == 3);
    var dustCol: vec4<f32> = rgb_color * ((dustMin - 1.0) * rng + 1.0) * f32(ptype == 2);
    var gridFactor: vec4<f32> = (rgb_color + gridCol) * f32(ptype == 0);
    var allOther: vec4<f32> = rgb_color * f32(ptype != 3 && ptype != 2);

    rgb_color = watrCol + dustCol + gridFactor + allOther;
    rgb_color.a = 1.0;//f32(ptype != 0);

    var real_coord = in.texture_coord;
    real_coord.x -= 2.0;
    rgb_color = (textureSample(gui_text, gui_sampl, real_coord) * f32(in.texture_coord.x > 1.0)) +
                rgb_color * f32(in.texture_coord.x <= 1.0);

    return rgb_color;
}
